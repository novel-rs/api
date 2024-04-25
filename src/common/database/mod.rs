mod entity;
mod migration;

use std::{io::Cursor, path::PathBuf, time::Duration};

use async_compression::tokio::{bufread::ZstdDecoder, write::ZstdEncoder};
use chrono::NaiveDateTime;
use image::{io::Reader, DynamicImage};
use sea_orm::{ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use tracing::{error, info};
use url::Url;

use crate::{ChapterInfo, Error};
use entity::{Image, Text};
use migration::{Migrator, MigratorTrait};

#[must_use]
pub(crate) struct NovelDB {
    db: DatabaseConnection,
}

#[must_use]
#[derive(Debug, PartialEq)]
pub(crate) enum FindTextResult {
    Ok(String),
    None,
    Outdate,
}

#[must_use]
#[derive(Debug, PartialEq)]
pub(crate) enum FindImageResult {
    Ok(DynamicImage),
    None,
}

impl NovelDB {
    const DB_NAME: &'static str = "novel.db";

    pub(crate) async fn new(app_name: &str) -> Result<Self, Error> {
        let db_path = NovelDB::db_path(app_name)?;

        if fs::try_exists(&db_path).await? {
            info!("The database file is located at `{}`", db_path.display());
        } else {
            info!(
                "The database file will be created at `{}`",
                db_path.display()
            );

            fs::create_dir_all(db_path.parent().unwrap()).await?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let mut opt = ConnectOptions::new(&db_url);
        opt.connect_timeout(Duration::from_secs(10));

        let mut db = Database::connect(opt).await?;
        if Migrator::up(&db, None).await.is_err() {
            error!("The file may not be a database, try recreating it");

            db.close().await?;

            let backup_path = db_path.with_extension("backup");
            fs::rename(&db_path, &backup_path).await?;
            info!("The file has been backed up to `{}`", backup_path.display());

            db = Database::connect(ConnectOptions::new(&db_url)).await?;
            Migrator::up(&db, None).await?;
        }

        Ok(Self { db })
    }

    #[cfg(test)]
    pub(crate) async fn drop(&self) -> Result<(), Error> {
        Ok(Migrator::down(&self.db, None).await?)
    }

    pub(crate) async fn find_text(&self, info: &ChapterInfo) -> Result<FindTextResult, Error> {
        match Text::find_by_id(info.id).one(&self.db).await? {
            Some(model) => {
                let saved_data_time = model.date_time;
                let time = NovelDB::get_time(info);

                if time.is_some()
                    && saved_data_time.is_some()
                    && saved_data_time.unwrap() < time.unwrap()
                {
                    Ok(FindTextResult::Outdate)
                } else {
                    Ok(FindTextResult::Ok(unsafe {
                        String::from_utf8_unchecked(zstd_decompress(&model.content).await?)
                    }))
                }
            }

            None => Ok(FindTextResult::None),
        }
    }

    pub(crate) async fn insert_text<T>(&self, info: &ChapterInfo, text: T) -> Result<(), Error>
    where
        T: AsRef<str>,
    {
        let model = entity::text::ActiveModel {
            id: sea_orm::Set(info.id),
            date_time: sea_orm::Set(NovelDB::get_time(info)),
            content: sea_orm::Set(zstd_compress(text.as_ref().as_bytes()).await?),
        };
        model.insert(&self.db).await?;

        Ok(())
    }

    pub(crate) async fn update_text<T>(&self, info: &ChapterInfo, text: T) -> Result<(), Error>
    where
        T: AsRef<str>,
    {
        let model = entity::text::ActiveModel {
            id: sea_orm::Set(info.id),
            date_time: sea_orm::Set(NovelDB::get_time(info)),
            content: sea_orm::Set(zstd_compress(text.as_ref().as_bytes()).await?),
        };
        model.update(&self.db).await?;

        Ok(())
    }

    pub(crate) async fn find_image(&self, url: &Url) -> Result<FindImageResult, Error> {
        let model = Image::find_by_id(url.to_string()).one(&self.db).await?;

        match model {
            Some(model) => {
                let bytes = zstd_decompress(&model.content).await?;
                let image = Reader::new(Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?;

                Ok(FindImageResult::Ok(image))
            }
            None => Ok(FindImageResult::None),
        }
    }

    pub(crate) async fn insert_image<T>(&self, url: &Url, bytes: T) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
    {
        let model = entity::image::ActiveModel {
            url: sea_orm::Set(url.to_string()),
            content: sea_orm::Set(zstd_compress(bytes).await?),
        };
        model.insert(&self.db).await?;

        Ok(())
    }

    fn db_path(app_name: &str) -> Result<PathBuf, Error> {
        let mut db_path = crate::data_dir_path(app_name)?;
        db_path.push(NovelDB::DB_NAME);

        Ok(db_path)
    }

    fn get_time(info: &ChapterInfo) -> Option<NaiveDateTime> {
        if info.update_time.is_some() {
            info.update_time
        } else {
            info.create_time
        }
    }
}

async fn zstd_decompress<T>(data: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let mut reader = ZstdDecoder::new(BufReader::new(data.as_ref()));
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;

    Ok(buf)
}

async fn zstd_compress<T>(data: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let mut writer = ZstdEncoder::new(Vec::new());
    writer.write_all(data.as_ref()).await?;
    writer.shutdown().await?;

    let mut res = writer.into_inner();
    res.flush().await?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn zstd() -> Result<(), Error> {
        let data = "test-data";

        let compressed_data = zstd_compress(data).await?;
        let decompressed_data = zstd_decompress(compressed_data).await?;

        assert_eq!(data.as_bytes(), decompressed_data.as_slice());

        Ok(())
    }

    #[tokio::test]
    async fn db() -> Result<(), Error> {
        let app_name = "test-app";
        let contents = "test-contents";

        let db = NovelDB::new(app_name).await?;

        let chapter_info_old = ChapterInfo {
            id: 0,
            update_time: Some(NaiveDateTime::from_str("2020-07-08T15:25:15")?),
            ..Default::default()
        };

        let chapter_info_new = ChapterInfo {
            id: 0,
            update_time: Some(NaiveDateTime::from_str("2020-07-08T15:25:17")?),
            ..Default::default()
        };

        assert_eq!(db.find_text(&chapter_info_new).await?, FindTextResult::None);

        db.insert_text(&chapter_info_old, contents).await?;
        assert_eq!(
            db.find_text(&chapter_info_new).await?,
            FindTextResult::Outdate
        );

        db.update_text(&chapter_info_new, contents).await?;

        if let FindTextResult::Ok(result) = db.find_text(&chapter_info_new).await? {
            assert_eq!(result, contents);
        } else {
            panic!("Incorrect database query result");
        }

        db.drop().await?;

        Ok(())
    }
}

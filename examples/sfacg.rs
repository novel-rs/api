use anyhow::Result;

use novel_api::{Client, Options, SfacgClient, WordCountRange};

#[tokio::main]
async fn main() -> Result<()> {
    let client = SfacgClient::new().await?;

    assert!(client.logged_in().await?);

    let user_info = client.user_info().await?;
    println!("{user_info:#?}");

    let money = client.money().await?;
    println!("{money}");

    client.sign().await?;

    let bookshelf_infos = client.bookshelf_infos().await?;
    println!("{bookshelf_infos:#?}");

    let novel_id = 263060;

    let novel_info = client.novel_info(novel_id).await?.unwrap();
    println!("{novel_info:#?}");

    let volume_infos = client.volume_infos(novel_id).await?.unwrap();
    println!("{volume_infos:#?}");

    let content_infos = client
        .content_infos(&volume_infos[0].chapter_infos[0])
        .await?;
    println!("{content_infos:#?}");

    let categories = client.categories().await?;
    println!("{categories:#?}");

    let tags = client.tags().await?;
    println!("{tags:#?}");

    let options = Options {
        word_count: Some(WordCountRange::RangeFrom(50_0000..)),
        ..Default::default()
    };
    let novels = client.search_infos(&options, 0, 12).await?;
    println!("{novels:#?}");

    Ok(())
}

use once_cell::sync::OnceCell;
use tracing::error;
use uuid::Uuid;

#[must_use]
pub(crate) fn uid() -> &'static String {
    static UID: OnceCell<String> = OnceCell::new();

    UID.get_or_init(|| {
        let mut result = match machine_uid::get() {
            Ok(uid) if !uid.is_empty() => uid,
            _ => {
                error!("Machine id generation failed, use uuid instead");
                Uuid::new_v4().to_string()
            }
        };

        result.make_ascii_uppercase();
        result
    })
}

use dialoguer::{theme::ColorfulTheme, Input, Password};

use crate::Error;

pub fn input<T>(prompt: T) -> Result<String, Error>
where
    T: AsRef<str>,
{
    Ok(Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.as_ref())
        .interact_text()?)
}

pub fn password<T>(prompt: T) -> Result<String, Error>
where
    T: AsRef<str>,
{
    Ok(Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.as_ref())
        .interact()?)
}

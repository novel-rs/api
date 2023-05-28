use keyring::Entry;

use crate::Error;

/// Access the Keyring of the platform
#[must_use]
pub struct Keyring {
    entry: Entry,
}

impl Keyring {
    /// Create a Keyring
    pub fn new<T, E>(app_name: T, username: E) -> Result<Self, Error>
    where
        T: AsRef<str>,
        E: AsRef<str>,
    {
        let service = format!("novel-{}", app_name.as_ref());
        let entry = Entry::new(&service, username.as_ref())?;

        Ok(Self { entry })
    }

    /// Get password
    pub fn get_password(&self) -> Result<String, Error> {
        Ok(self.entry.get_password()?)
    }

    /// Set password
    pub fn set_password<T>(&self, password: T) -> Result<(), Error>
    where
        T: AsRef<str>,
    {
        Ok(self.entry.set_password(password.as_ref())?)
    }

    /// Delete password
    pub fn delete_password(&self) -> Result<(), Error> {
        Ok(self.entry.delete_password()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn keyring() -> Result<(), Error> {
        if !is_ci::uncached() {
            let password = "test-username";
            let keyring = Keyring::new("test", password)?;

            keyring.set_password(password)?;
            assert_eq!(keyring.get_password()?, password);

            keyring.delete_password()?;
        }

        Ok(())
    }
}

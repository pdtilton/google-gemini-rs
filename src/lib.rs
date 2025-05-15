pub mod client;
pub mod google;

#[cfg(test)]
mod test {
    use dotenv::dotenv;
    use std::{env, path::Path};
    use thiserror::Error;

    use crate::{
        client::{self, Client},
        google,
    };

    const GEMINI_API_ENV_KEY: &str = "GEMINI_API_KEY";
    const GEMINI_MODEL_ENV_KEY: &str = "GEMINI_MODEL";

    const TUX_IMAGE_PATH: &str = "tests/images/tux.png";

    #[derive(Debug, Error)]
    enum Error {
        #[error(transparent)]
        DotEnv(#[from] dotenv::Error),
        #[error(transparent)]
        Client(#[from] client::Error),
        #[error(transparent)]
        Var(#[from] env::VarError),
        #[error(transparent)]
        Io(#[from] std::io::Error),
        #[error(transparent)]
        Google(#[from] google::Error),
    }

    async fn client() -> Result<Client, Error> {
        dotenv()?;
        let key = env::var(GEMINI_API_ENV_KEY)?;
        let model = env::var(GEMINI_MODEL_ENV_KEY)?;

        Ok(Client::new(&model.try_into()?, &key).with_defaults().await)
    }

    #[tokio::test]
    async fn basic_query() -> Result<(), Error> {
        let mut client = client().await?;
        let response = client
            .send_text("I'm new to AI, so introduce yourself.")
            .await?;
        println!(
            "{}",
            response.extract_text().expect("Expected text output.")
        );
        let response = client.send_text("Hello.").await?;
        println!(
            "{}",
            response.extract_text().expect("Expected text output.")
        );
        Ok(())
    }

    #[tokio::test]
    async fn image_query() -> Result<(), Error> {
        let mut client = client().await?;
        let response = client
            .send_text("Generate a thumbnail sized picture of a capybara.")
            .await?;
        println!("{:?}", response.extract_images());
        response
            .extract_images()
            .first()
            .expect("Expected image output(s).");
        Ok(())
    }

    #[tokio::test]
    async fn image_and_text_query() -> Result<(), Error> {
        let mut client = client().await?;
        let response = client
            .send_text("Your role is an artists that upgrades logos.")
            .await?;
        println!(
            "{:?}",
            response.extract_text().expect("Expected text result.")
        );
        let pic = Path::new(TUX_IMAGE_PATH);
        let response = client.send_image(Some("Here is an image of the linux mascot, tux.  Add the words linux to the background".to_string()), &pic).await?;
        println!("{:?}", response.extract_text());
        println!(
            "{:?}",
            response
                .extract_images()
                .first()
                .expect("Expected image output(s)")
        );

        let response = client
            .send_text("What type of animal is in the image you sent?")
            .await?;
        println!(
            "{:?}",
            response.extract_text().expect("Expected text result.")
        );
        Ok(())
    }
}

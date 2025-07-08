pub mod client;
pub mod google;

pub use rust_mcp_sdk;

#[cfg(test)]
mod test {
    use dotenv::dotenv;
    use std::{env, path::Path};
    use thiserror::Error;

    use crate::{
        client::{self, Client},
        google::{self, common::Modality},
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

        Ok(Client::new(&model.as_str().try_into()?, &key)
            .await?
            .with_defaults())
    }

    #[tokio::test]
    async fn basic_query() -> Result<(), Error> {
        let mut client = client().await?;
        let response = client
            .send_text("I'm new to AI, so introduce yourself.")
            .await?;
        println!("{}", response.text().expect("Expected text output."));
        let response = client.send_text("Hello.").await?;
        println!("{}", response.text().expect("Expected text output."));
        let response = client.send_text("Do you have a name?").await?;
        println!("{}", response.text().expect("Expected text output."));
        Ok(())
    }

    #[tokio::test]
    async fn image_query() -> Result<(), Error> {
        let mut client = client().await?;

        let response = client
            .send_text("Generate a thumbnail sized picture of a capybara.")
            .await?;
        println!("Image response: {:?}", response.images());
        if client.model.output.contains(&Modality::Image) {
            response
                .images()
                .first()
                .expect("Expected image output(s).");
        }
        println!("Text response: {:?}", response.text());
        Ok(())
    }

    #[tokio::test]
    async fn image_and_text_query() -> Result<(), Error> {
        let mut client = client().await?;
        let response = client
            .send_text("Your role is an artists that upgrades logos.")
            .await?;

        println!("{:?}", response.text().expect("Expected text result."));

        let pic = Path::new(TUX_IMAGE_PATH);
        let response = client.send_image_file(Some("Here is an image of the linux mascot, tux.  Add the words linux to the background".to_string()), &pic).await?;

        println!("Response text: {:?}", response.text());

        if client.model.output.contains(&Modality::Image) {
            println!(
                "{:?}",
                response.images().first().expect("Expected image output(s)")
            );
        }

        let response = client
            .send_text("What type of animal is in the image you sent?")
            .await?;

        let text = response.text().expect("Expected text result.");

        println!("Response check text: {}", text);

        assert!(text.contains("penguin"));

        Ok(())
    }
}

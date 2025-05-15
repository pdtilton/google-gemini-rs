A unofficial simple Rust wrapper around Google's Gemini API.

Provides statically typed request and response objects as well as an easy to use
client.

### API Key

You may use a `.env` file for loading your Gemini API key and model using:

```bash
GEMINI_API_KEY="MY_KEY_VALUE"
GEMINI_MODEL="gemini-2.0-flash"
```

but this is not enforced.  It's up to the end-user to load their keys.

### Example Usage

```rust,no_run
    dotenv()?;

    let key = env::var(GEMINI_API_ENV_KEY)?;
    let model = env::var(GEMINI_MODEL_ENV_KEY)?;

    let mut client = Client::new(&model.try_into()?, &key).with_defaults().await;

    let response = client
        .send_text("Your role is an artists that upgrades logos.")
        .await?;

    println!("{:?}", response.extract_text().expect("Expected text result."));

    let pic = BASE64_URL_SAFE.encode(&tokio::fs::read(TUX_IMAGE_PATH).await?);

    let response = client.send_image_bytes(Some("Here is an image of the linux mascot, tux.  Add the words 'linux' to the background".to_string()), "image/png", &pic).await?;

    println!("{:?}", response.extract_text());

    println!("{:?}", response.extract_images().first().expect("Expected image output(s)"));

    let response = client
        .send_text("What type of animal is in the image you sent?")
        .await?;

    println!("{:?}", response.extract_text().expect("Expected text result."));
```

### Commercial Support

Commercial support may be obtained through Tilton Technologies, LLC at https://tiltontechnologies.com.
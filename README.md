A unofficial simple Rust wrapper around Google's Gemini API.

Provides statically typed request and response objects as well as an easy to use
client.

This is a work-in-progress as we become more familiar with the Google Gemini API, so
expect major changes between minor versions.  At 1.0.0 we will have stable APIs.

### API Key

You may use a `.env` file for loading your Gemini API key and model using:

```bash
GEMINI_API_KEY="MY_KEY_VALUE"
GEMINI_MODEL="gemini-2.0-flash"
```

but this is not enforced.  It's up to the end-user to load their keys.

### Gemini Model Names

Gemini models are still in flux.  The schema at https://ai.google.dev/gemini-api/docs/models#model-versions isn't
closely followed by google, so we're doing our best to provide easy configurations.

We support "gemini-2.0-flash-exp-image-generation", "gemini-2.0-flash", "gemini-2.5-flash", "gemini-2.5-flash-lite", and "gemini-2.5-pro".  We
also attempt to support preview versions by allowing "-preview*" to be appended to the name.  We do this so that we can infer the supported
input and output modalities, but these defaults may be overridden (see below).

Models may be instantiated by str

```rust
    let model = "gemini-2.5-pro";

    let mut client = Client::new(&model.try_into()?, &key).await?.with_defaults();
```

or manually if you want more fine-grained control.

```rust
    let key = env::var(GEMINI_API_ENV_KEY)?;

    let client = Client::new(&GoogleModel::new(GoogleModelVariant::Gemini25FlashLight, Some("-preview-06-17".to_string())), key).await?;
```

### Example Usage

```rust
    dotenv()?;

    let key = env::var(GEMINI_API_ENV_KEY)?;
    let model = env::var(GEMINI_MODEL_ENV_KEY)?;

    let mut client = Client::new(&model.as_str().try_into()?, &key).await?.with_defaults();

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

### Output Modalities

Model output modalities are inferred, but they can be overridden by using the `Client::with_options`.  This is particularly useful when
using Text To Speech (tts) version of models which support it.

### Commercial Support

Commercial support may be obtained through Tilton Technologies, LLC at https://tiltontechnologies.com.
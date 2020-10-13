use regex::Regex;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::process::exit;

// CLI inputs
struct Input {
    input_language: String,
    output_language: String,
    text: String,
}

// For deserializing API response from google
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct Translated {
    translatedText: String,
}

#[derive(Deserialize)]
struct Translations {
    translations: Vec<Translated>,
}

// This `derive` requires the `serde` dependency.
#[derive(Deserialize)]
struct Ip {
    data: Translations,
}

fn get_optional_env_var(key: &str) -> String {
    return match env::var_os(key) {
        Some(val) => match val.into_string() {
            Ok(val) => val,
            _ => "".to_string(),
        },
        None => "".to_string(),
    };
}

fn print_help() {
    println!(
        "
To translate something something using google translate, use the format
`google-translate -i <input_language> -o <output_language> <text to translate>.

You may also provide the input language with the environment variable GT_INPUT_LANGUAGE
and output language with environment variable GT_OUTPUT_LANGUAGE.

This requires an environment variable GOOGLE_ACCESS_KEY which can be retrieved with `gcloud auth application-default print-access-token`

The allowed languages are:

Afrikaans - af
Albanian - sq
Amharic - am
Arabic - ar
Armenian - hy
Azerbaijani - az
Basque - eu
Belarusian - be
Bengali - bn
Bosnian - bs
Bulgarian - bg
Catalan - ca
Cebuano - ceb (ISO-639-2)
Chinese (Simplified) - zh-CN or zh (BCP-47)
Chinese (Traditional) - zh-TW (BCP-47)
Corsican - co
Croatian - hr
Czech - cs
Danish - da
Dutch - nl
English - en
Esperanto - eo
Estonian - et
Finnish - fi
French - fr
Frisian - fy
Galician - gl
Georgian - ka
German - de
Greek - el
Gujarati - gu
Haitian Creole - ht
Hausa - ha
Hawaiian - haw (ISO-639-2)
Hebrew - he or iw
Hindi - hi
Hmong - hmn (ISO-639-2)
Hungarian - hu
Icelandic - is
Igbo - ig
Indonesian - id
Irish - ga
Italian - it
Japanese - ja
Javanese - jv
Kannada - kn
Kazakh - kk
Khmer - km
Kinyarwanda - rw
Korean - ko
Kurdish - ku
Kyrgyz - ky
Lao - lo
Latin - la
Latvian - lv
Lithuanian - lt
Luxembourgish - lb
Macedonian - mk
Malagasy - mg
Malay - ms
Malayalam - ml
Maltese - mt
Maori - mi
Marathi - mr
Mongolian - mn
Myanmar (Burmese) - my
Nepali - ne
Norwegian - no
Nyanja (Chichewa) - ny
Odia (Oriya) - or
Pashto - ps
Persian - fa
Polish - pl
Portuguese (Portugal, Brazil) - pt
Punjabi - pa
Romanian - ro
Russian - ru
Samoan - sm
Scots Gaelic - gd
Serbian - sr
Sesotho - st
Shona - sn
Sindhi - sd
Sinhala (Sinhalese) - si
Slovak - sk
Slovenian - sl
Somali - so
Spanish - es
Sundanese - su
Swahili - sw
Swedish - sv
Tagalog (Filipino) - tl
Tajik - tg
Tamil - ta
Tatar - tt
Telugu - te
Thai - th
Turkish - tr
Turkmen - tk
Ukrainian - uk
Urdu - ur
Uyghur - ug
Uzbek - uz
Vietnamese - vi
Welsh - cy
Xhosa - xh
Yiddish - yi
Yoruba - yo
Zulu - zu"
    );
    exit(1);
}

fn parse_input() -> Input {
    let allowed_languages: Vec<&str> = vec![
        "af", "sq", "am", "ar", "hy", "az", "eu", "be", "bn", "bs", "bg", "ca", "ceb", "zh-CN",
        "zh", "zh-TW", "co", "hr", "cs", "da", "nl", "en", "eo", "et", "fi", "fr", "fy", "gl",
        "ka", "de", "el", "gu", "ht", "ha", "haw", "he", "iw", "hi", "hmn", "hu", "is", "ig", "id",
        "ga", "it", "ja", "jv", "kn", "kk", "km", "rw", "ko", "ku", "ky", "lo", "la", "lv", "lt",
        "lb", "mk", "mg", "ms", "ml", "mt", "mi", "mr", "mn", "my", "ne", "no", "ny", "or", "ps",
        "fa", "pl", "pt", "pa", "ro", "ru", "sm", "gd", "sr", "st", "sn", "sd", "si", "sk", "sl",
        "so", "es", "su", "sw", "sv", "tl", "tg", "ta", "tt", "te", "th", "tr", "tk", "uk", "ur",
        "ug", "uz", "vi", "cy", "xh", "yi", "yo", "zu",
    ];

    // join arguments into one string
    let args: String = env::args().skip(1).collect::<Vec<String>>().join(" ");

    // Regex to check for possible input after -i, possible output after -o, the text to translate
    let re = Regex::new(
        r"^(-i (?P<input_language>[a-z]+))?(\s*-o (?P<output_language>[a-z]+))?(?P<text>.*)$",
    )
    .unwrap();

    // check if user wants help
    if args.contains("--help") {
        print_help();
    }

    // First check if an environment variable defines the input and output, but they can override these
    let mut input_language = get_optional_env_var("GT_INPUT_LANGUAGE");
    let mut output_language = get_optional_env_var("GT_OUTPUT_LANGUAGE");

    let mut text = String::new();

    match re.captures(&args) {
        Some(val) => {
            match val.name("input_language") {
                Some(val) => input_language = val.as_str().to_string(),
                None => (),
            };
            match val.name("output_language") {
                Some(val) => output_language = val.as_str().to_string(),
                None => (),
            };
            match val.name("text") {
                Some(val) => text = val.as_str().trim().to_string(),
                None => (),
            };
        }
        None => (),
    }

    // Check that input and output languages provided and are allowed
    if input_language.len() == 0 {
        println!("No input language provided. Type --help to see allowed languages");
        exit(1);
    }

    if output_language.len() == 0 {
        println!("No output language provided. Type --help to see allowed languages");
        exit(1);
    }

    if !allowed_languages.iter().any(|&i| i == input_language) {
        println!("Input language is not allowed. Type --help to see allowed languages");
        exit(1);
    }
    if !allowed_languages.iter().any(|&i| i == output_language) {
        println!("Output language is not allowed. Type --help to see allowed languages");
        exit(1);
    }

    Input {
        input_language: input_language,
        output_language: output_language,
        text: text,
    }
}

fn translate(input: Input) {
    let access_key = get_optional_env_var("GOOGLE_ACCESS_KEY");
    if access_key.len() == 0 {
        println!("A Google access key is required. See this for how to create one: https://cloud.google.com/translate/docs/setup");
        exit(1);
    }

    // Build body for http call
    let mut body = HashMap::new();
    body.insert("source", input.input_language);
    body.insert("target", input.output_language);
    body.insert("q", input.text);

    // Make call
    let client = reqwest::blocking::Client::new();
    let res: Result<Ip, reqwest::Error> = client
        .post("https://translation.googleapis.com/language/translate/v2")
        .header("Authorization", format!("Bearer {}", access_key))
        .json(&body)
        .send()
        .unwrap()
        .json();

    match res {
        Ok(res) => println!("{}", res.data.translations[0].translatedText),
        Err(e) => println!("There was the following error with the API call: {}", e),
    }
}

fn main() {
    let input = parse_input();
    translate(input);
}

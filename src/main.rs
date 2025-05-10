use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::io::{self, Write};
use url::form_urlencoded;

const STANDARD_EMAIL_REGEX: &str = r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#;

fn get_user_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn fetch_website_content(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Fetching content...");
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch URL: {}", response.status()).into());
    }
    let body = response.text()?;
    Ok(body)
}

fn extract_details(content: &str, regex_pattern: &str) -> HashSet<String> {
    let re = match Regex::new(regex_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Invalid regex pattern '{}': {}", regex_pattern, e);
            return HashSet::new();
        }
    };
    re.find_iter(content)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

fn is_date(input: &str) -> bool {
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(day), Ok(month), Ok(year)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            return day >= 1 && day <= 31 && month >= 1 && month <= 12 && year >= 1900 && year <= 2100;
        }
    }
    false
}

fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let website_url = get_user_input("Enter the website URL to scrape: ")?;
    if website_url.is_empty() {
        eprintln!("No URL provided. Exiting.");
        return Ok(());
    }

    let mut phone_regex_str = get_user_input(
        "Enter the regex for phone numbers (e.g., \\d{3}-\\d{3}-\\d{4}): ",
    )?;
    if phone_regex_str.is_empty() {
        phone_regex_str = r"(?:\+?\d{1,4}[-.\s]?)?$?\d{1,4}$?[-.\s]?\d{1,4}[-.\s]?\d{1,9}".to_string();
    }

    let html_content = match fetch_website_content(&website_url) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error fetching website content: {}", e);
            return Ok(());
        }
    };

    let mut phone_numbers = HashSet::new();
    if !phone_regex_str.is_empty() {
        println!("\nExtracting phone numbers using regex: {}", phone_regex_str);
        phone_numbers = extract_details(&html_content, &phone_regex_str);
    }
    phone_numbers.retain(|number| {
        !number.contains(".") 
        && number.len() >= 10
        && !is_date(number)
        && number.len() <= 16
        
    });

    println!("\nExtracting emails using regex: {}", STANDARD_EMAIL_REGEX);
    let emails = extract_details(&html_content, STANDARD_EMAIL_REGEX);

    println!("\nFound {} unique phone number(s):", phone_numbers.len());
    if phone_numbers.is_empty() {
        println!("No phone numbers found.");
    } 
    else {
        for phone in &phone_numbers {
            println!("- {}", phone);
        }
    }

    println!("Found {} unique email(s):", emails.len());
    if emails.is_empty() {
        println!("No emails found.");
    } 
    else {
        for email in &emails {
            println!("- {}", email);
        }
    }

    if !phone_numbers.is_empty() {
        println!("\n--- WhatsApp Message Generation ---");
        println!("Note: The Phone Numbers will be trimed to 10 digits to generate the link.");
        let send_messages_choice =
            get_user_input("Do you want to generate WhatsApp message links? (y/n): ")?
                .to_lowercase();

        if send_messages_choice == "yes" || send_messages_choice == "y" {
            let mut custom_message = get_user_input("Enter your custom message: ")?;
            if custom_message.is_empty() {
                println!("No message provided. Default message used:\nHi, Batscram needs your help.");
                custom_message="Hi, Batscram needs your help".to_string();
            }
            println!("\nMake sure you have logged in.\nGenerated Links (ctrl+click to open in WhatsApp):");
            let encoded_message: String = form_urlencoded::byte_serialize(custom_message.as_bytes()).collect();

            for phone in &phone_numbers {
                    // This part might require adjustment based on how your regex captures numbers.
                let cleaned_phone: String = phone
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .rev()
                    .take(10)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();

                if !cleaned_phone.is_empty() {
                         // IMPORTANT: You might need to prepend country codes if not captured by regex
                         // and ensure the number format is valid for wa.me links.
                         // For example, if your regex captures "123-456-7890" for US, you might need "11234567890".
                    println!("WhatsApp message link for {}: https://wa.me/{}?text={}", cleaned_phone, cleaned_phone, encoded_message);
                } 
                else {
                   println!("Could not generate link for '{}' (no digits found after cleaning)", phone);
                }
            }
            println!("\nVerify the numbers before sending.");
        }
        else {
            println!("Skipping message link generation.");
        }
    }

    println!("\nBEEP...\nEND.");
    Ok(())
}

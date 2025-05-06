use reqwest::blocking::get;
use regex::Regex;
use scraper::{Html, Selector}; // Add Selector import
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::io;
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run <url>");
        eprintln!("Example: cargo run https://www.rust-lang.org");
        return Ok(()); // Exit gracefully
    }
    let url = &args[1];
    println!("Scraping: {}", url);

    let html_content = match fetch_html(url) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error fetching URL {}: {}", url, e);
            eprintln!("Provide complete link with http/https header");
            return Err(Box::new(e)); // Return the specific error
        }
    };

    let (phone_numbers, emails) = match scrape_contact_info(&html_content) {
        Ok((phones, emails)) => (phones, emails),
        Err(e) => {
            eprintln!("Error scraping contact info: {}", e);
            return Err(e); // Return the specific error
        }
    };


    println!("\n--- Contact Information Found ---");

    if phone_numbers.is_empty() {
        println!("No unique phone numbers discovered.");
    } else {
        println!("Unique Phone Numbers:");
        for number in phone_numbers {
            println!("- {}", number);
        }
    }

    println!(); // Add a blank line

    if emails.is_empty() {
        println!("No unique email addresses discovered.");
    } else {
        println!("Unique Email Addresses:");
        for email in emails {
            println!("- {}", email);
        }
    }

    Ok(())
}

/// Retrieves the HTML content from the specified URL.
fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    // if !url.starts_with("http://") && !url.starts_with("https://") {
    //     // This is not a reqwest::Error, so we need to handle it differently
    //     // or return a custom error type if more sophisticated error handling is needed.
    //     // For simplicity here, we'll let reqwest handle potential malformed URLs
    //     // when `get` is called, or rely on the pattern match above.
    // }
    let response = get(url)?;
    response.text()
}
fn is_date(input: &str) -> bool {
    // Check if the input is a date in the format dd-mm-yyyy
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

/// Extracts phone numbers and email addresses from HTML content.
fn scrape_contact_info(html_content: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    // This regex attempts to capture common patterns 
    // You might need to adjust this based on the specific formats you expect.
    println!("Enter phone number Regex <To use default, press Enter>: ");

    let mut phone_regex_input = String::new();
    io::stdin().read_line(&mut phone_regex_input)
        .expect("Failed to read line");

    let phone_regex = if phone_regex_input.trim().is_empty() {
        Regex::new(r"(?:\+?\d{1,4}[-.\s]?)?$?\d{1,4}$?[-.\s]?\d{1,4}[-.\s]?\d{1,9}")
            .expect("Phone Regex Dead")
    } else {
        Regex::new(&phone_regex_input.trim())
            .expect("Bad Phone Regex")
    };
    let email_regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?;

    let mut phone_numbers: HashSet<String> = HashSet::new();
    let mut email_addresses: HashSet<String> = HashSet::new();

    for mat in phone_regex.find_iter(html_content) {
        phone_numbers.insert(mat.as_str().to_string());
    }
    for mat in email_regex.find_iter(html_content) {
        email_addresses.insert(mat.as_str().to_string());
    }

    phone_numbers.retain(|number| {
        !number.contains(".") 
        && number.len() >= 10
        && !is_date(number)
        && number.len() <= 16
        
    });


    let mut sorted_phones: Vec<String> = phone_numbers.into_iter().collect();
    sorted_phones.sort();

    let mut sorted_emails: Vec<String> = email_addresses.into_iter().collect();
    sorted_emails.sort();


    Ok((sorted_phones, sorted_emails))
}

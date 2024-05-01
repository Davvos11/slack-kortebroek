use std::env;
use std::process::exit;
use html_escape::decode_html_entities;
use reqwest::blocking::Client;
use tl::ParserOptions;
use anyhow::{anyhow, Context, Result};

fn main() -> Result<()> {
    let slack_url = env::var("SLACK_URL").unwrap_or_else(|_| {
        eprintln!("Please provide a Slack Incoming Webhook url in the SLACK_URL env variable");
        exit(1);
    });
    let client = Client::new();
    let (verdict, stats) = get_korte_broek(&client)
        .context("Failed to fetch or parse kanikeenkortebroekaan.nl data")?;
    send_to_slack(&client, &slack_url, verdict, &stats)
        .context("Failed to send Slack message")?;
    Ok(())
}

fn get_korte_broek(client: &Client) -> Result<(bool, Vec<String>)> {
    let response = client.get("https://www.kanikeenkortebroekaan.nl/").send()?.text()?;
    let dom = tl::parse(response.as_str(), ParserOptions::default())?;
    let parser = dom.parser();
    let body_class =
        dom.query_selector("body").and_then(|mut b| b.next()).context("Body element not found")?
            .get(parser).and_then(|b| b.as_tag()).context("Could not get body")?
            .attributes().class().context("Could not get class")?.as_utf8_str();
    let verdict;
    if body_class.contains("ja") {
        verdict = true;
    } else if body_class.contains("nee") {
        verdict = false;
    } else {
        return Err(anyhow!("Body class did not include 'ja' or 'nee'"));
    };


    let stats: Result<Vec<String>> = dom.get_elements_by_class_name("data")
        .map(|n| {
            let node = n.get(parser).context("Failed to get data elements")?;
            let text = node.inner_text(parser);
            Ok(decode_html_entities(&text).to_string())
        })
        .collect();
    Ok((verdict, stats?))
}

fn send_to_slack(client: &Client, slack_url: &String, verdict: bool, stats: &[String]) -> Result<()> {
    let emoji = if verdict { ":ja_: dat kan!" } else { ":nee_: dat kan niet..." };
    let text = format!(":goeiemorge: *Kan ik vandaag een korte broek aan?*\n{emoji}\n{}", stats.join(", "));
    client.post(slack_url)
        .body(format!("{{'text': '{text}' }}"))
        .send()?;
    Ok(())
}

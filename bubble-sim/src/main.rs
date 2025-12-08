use bubble_core::BubbleClient;
use anyhow::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Client (Load keys from same file as TUI)
    println!("Initializing Simulator Client...");
    let client = BubbleClient::new(None).await?;
    let my_pubkey = client.get_own_pubkey().await;
    println!("Simulator Pubkey: {}", my_pubkey);

    // 2. Fetch latest timeline
    println!("Fetching latest events...");
    let events = client.get_timeline(5).await?;
    
    if events.is_empty() {
        println!("No events found to verify.");
        return Ok(());
    }

    // 3. Pick the latest event
    let target_event = &events[0];
    println!("Targeting event: {} from {}", target_event.id, target_event.pubkey);

    // 4. Publish Label (Kind 1985)
    // Label content: "This is FAKE" or "Verified"
    let label = "FAKE NEWS (Simulated)";
    println!("Publishing label: '{}'", label);
    
    let label_id = client.publish_label(target_event.id, label.to_string()).await?;
    println!("Label published! Event ID: {}", label_id);
    
    println!("Waiting for propagation...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    println!("Done. Run 'r' in TUI to refresh.");

    Ok(())
}

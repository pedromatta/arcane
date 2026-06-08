use arcane_core::db::schedule::{add_schedule_slot, list_schedule_slots, remove_schedule_slot};
use sqlx::SqlitePool;

pub async fn add_slot(pool: &SqlitePool, category_id: u32, time: &str, days: u8) {
    match add_schedule_slot(category_id, time, days, pool).await {
        Ok(_) => {
            println!(
                "Successfully added weekly schedule slot: Category ID {}, Time {}, Days bitmask {}.",
                category_id, time, days
            );
        }
        Err(e) => {
            eprintln!("Error adding schedule slot: {}", e);
        }
    }
}

pub async fn list_slots(pool: &SqlitePool) {
    match list_schedule_slots(pool).await {
        Ok(slots) => {
            if slots.is_empty() {
                println!("No weekly scheduled slots found.");
            } else {
                println!("+----+-------------+-------------+---------------+");
                println!("| ID | Category ID | Time of Day | Days Bitmask  |");
                println!("+----+-------------+-------------+---------------+");
                for slot in slots {
                    println!(
                        "| {:<2} | {:<11} | {:<11} | {:<13} |",
                        slot.id, slot.category_id, slot.time_of_day, slot.days_of_week
                    );
                }
                println!("+----+-------------+-------------+---------------+");
            }
        }
        Err(e) => {
            eprintln!("Error listing schedule slots: {}", e);
        }
    }
}

pub async fn remove_slot(pool: &SqlitePool, slot_id: u32) {
    match remove_schedule_slot(slot_id, pool).await {
        Ok(_) => {
            println!("Successfully removed schedule slot ID {}.", slot_id);
        }
        Err(e) => {
            eprintln!("Error removing schedule slot: {}", e);
        }
    }
}

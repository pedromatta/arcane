use arcane_core::db::schedule::{add_schedule_slot, list_schedule_slots_detail, remove_schedule_slot};
use sqlx::SqlitePool;

pub async fn add_slot(pool: &SqlitePool, category: &str, time: &str, days: &str) {
    match add_schedule_slot(category, time, days, pool).await {
        Ok(_) => {
            println!(
                "Successfully added weekly schedule slot for category '{}' at {} on ({}).",
                category, time, days
            );
        }
        Err(e) => {
            eprintln!("Error adding schedule slot: {}", e);
        }
    }
}

pub async fn list_slots(pool: &SqlitePool) {
    match list_schedule_slots_detail(pool).await {
        Ok(slots) => {
            if slots.is_empty() {
                println!("No weekly scheduled slots found.");
                return;
            }

            let weekdays = [
                ("Monday", 1),
                ("Tuesday", 2),
                ("Wednesday", 4),
                ("Thursday", 8),
                ("Friday", 16),
                ("Saturday", 32),
                ("Sunday", 64),
            ];

            let mut displayed_any = false;
            for (day_name, day_bit) in weekdays {
                // Filter and sort slots for the current day
                let mut day_slots: Vec<_> = slots
                    .iter()
                    .filter(|s| (s.days_of_week & day_bit) != 0)
                    .collect();
                
                if !day_slots.is_empty() {
                    day_slots.sort_by(|a, b| a.time_of_day.cmp(&b.time_of_day));
                    println!("{}:", day_name);
                    for slot in day_slots {
                        println!(
                            "  - Slot ID: {:<2} | {} | {}",
                            slot.id, slot.time_of_day, slot.category_name
                        );
                    }
                    println!();
                    displayed_any = true;
                }
            }

            if !displayed_any {
                println!("No active slots scheduled for any weekdays.");
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

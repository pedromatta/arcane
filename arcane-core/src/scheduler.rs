use crate::models::schedule_slot::ScheduleSlot;
use crate::models::schedule_override::ScheduleOverride;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineBlock {
    pub time_of_day: String, // "HH:MM"
    pub category_id: u32,
}

pub fn resolve_timeline(
    date: NaiveDate,
    slots: &[ScheduleSlot],
    overrides: &[ScheduleOverride],
) -> Vec<TimelineBlock> {
    // Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64
    let num_from_monday = date.weekday().number_from_monday();
    let weekday_bit = 1 << (num_from_monday - 1);

    // Collect all times from overrides and slots
    let mut times: Vec<String> = Vec::new();
    for ov in overrides {
        if !times.contains(&ov.time_of_day) {
            times.push(ov.time_of_day.clone());
        }
    }
    for slot in slots {
        if (slot.days_of_week as u32 & weekday_bit) != 0 && !times.contains(&slot.time_of_day) {
            times.push(slot.time_of_day.clone());
        }
    }

    // Sort times chronologically
    times.sort();

    let mut blocks = Vec::new();
    for time in times {
        // Check overrides first
        if let Some(ov) = overrides.iter().find(|o| o.time_of_day == time) {
            if let Some(cat_id) = ov.category_id {
                blocks.push(TimelineBlock {
                    time_of_day: time,
                    category_id: cat_id,
                });
            }
            // If category_id is None, it represents a rest block (skipped)
        } else if let Some(slot) = slots
            .iter()
            .find(|s| s.time_of_day == time && (s.days_of_week as u32 & weekday_bit) != 0)
        {
            blocks.push(TimelineBlock {
                time_of_day: time,
                category_id: slot.category_id,
            });
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_resolve_timeline_basic() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap(); // Monday (Bit 1)
        let slots = vec![
            ScheduleSlot {
                id: 1,
                category_id: 1,
                time_of_day: "08:00".to_string(),
                days_of_week: 31, // Mon-Fri
            },
            ScheduleSlot {
                id: 2,
                category_id: 2,
                time_of_day: "10:00".to_string(),
                days_of_week: 127, // Every day
            },
            ScheduleSlot {
                id: 3,
                category_id: 3,
                time_of_day: "14:00".to_string(),
                days_of_week: 64, // Sun only
            },
        ];

        let overrides = vec![
            ScheduleOverride {
                id: 1,
                category_id: Some(4), // Override 10:00 block to category 4
                override_date: date,
                time_of_day: "10:00".to_string(),
            },
            ScheduleOverride {
                id: 2,
                category_id: None, // Clear 08:00 block
                override_date: date,
                time_of_day: "08:00".to_string(),
            },
        ];

        let timeline = resolve_timeline(date, &slots, &overrides);

        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].time_of_day, "10:00");
        assert_eq!(timeline[0].category_id, 4);
    }
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Category {
    id: u32,
    name: String,
    default_minutes: u32,
    color: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let category = Category {
            id: 1,
            name: "Work".to_string(),
            default_minutes: 25,
            color: "#FF0000".to_string(),
        };
        assert_eq!(category.id, 1);
        assert_eq!(category.name, "Work");
        assert_eq!(category.default_minutes, 25);
        assert_eq!(category.color, "#FF0000");
    }
}

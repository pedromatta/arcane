CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    default_minutes INTEGER NOT NULL,
    color TEXT NOT NULL
);

CREATE TABLE schedule_slots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    time_of_day TEXT NOT NULL, -- Format: "HH:MM"
    days_of_week INTEGER NOT NULL, -- 7-bit bitmask (0-127)
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE TABLE schedule_overrides (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    override_date TEXT NOT NULL, -- Format: "YYYY-MM-DD"
    time_of_day TEXT NOT NULL, -- Format: "HH:MM"
    category_id INTEGER,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(override_date, time_of_day)
);

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    start_time TEXT NOT NULL, -- Format: ISO 8601 YYYY-MM-DD HH:MM:SS
    duration_minutes INTEGER NOT NULL,
    notes TEXT,
    rating INTEGER CHECK(rating >= 0 AND rating <= 5),
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
);

CREATE TABLE review_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    topic TEXT NOT NULL,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 0,
    next_review_date TEXT NOT NULL, -- Format: YYYY-MM-DD
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(category_id, topic)
);

CREATE INDEX idx_schedule_slots_days ON schedule_slots(days_of_week);
CREATE INDEX idx_schedule_slots_category ON schedule_slots(category_id);

CREATE INDEX idx_schedule_overrides_date ON schedule_overrides(override_date);

CREATE INDEX idx_sessions_time ON sessions(start_time);
CREATE INDEX idx_sessions_category ON sessions(category_id);

CREATE INDEX idx_review_next_date ON review_states(next_review_date);

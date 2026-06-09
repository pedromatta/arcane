# Arcane
## Routine planner utility developed in Rust

Arcane is a command-line interface (CLI) study routine planner and active scheduler. Built fto optimize daily learning cycle. It's a developer-focused offline tool, that includes:
    
* A local SQLite database to manage categories and schedule slots;
* An active timer engine using the SuperMemo-2 (SM-2) to eventually schedule recall reviews;
* A countdown timer loop with native desktop notifications;

## Installation

### Prerequisites

Ensure you have Rust (v1.75+ or newer) installed.

### Installing from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/pedromatta/arcane.git
   cd arcane
   ```

2. Build the workspace binaries:
   ```bash
   cargo build --release
   ```

3. The compiled binary will be located in `target/release/arcane-cli`. You can copy or symlink it to your PATH:
   ```bash
   cp target/release/arcane-cli /usr/local/bin/arcane
   ```

---

## Usage

### 1. Guided Setup Wizard

To initialize your settings, categories, and schedule slots interactively:
```bash
arcane setup
```

### 2. Category Management

* **Add a category**:
  ```bash
  arcane categories add --name "Rust Programming" --default-minutes 45 --color "magenta"
  ```
  Colors can be standard named colors (e.g., white, red, green, magenta), ANSI numbers (0-255), or custom hex codes (e.g., #ff5733).

* **List categories**:
  ```bash
  arcane categories list
  ```

* **Remove a category**:
  ```bash
  arcane categories remove --name "Rust Programming"
  ```
  If a category has logged sessions, it is archived to preserve history rather than hard deleted.

### 3. Weekly Schedule Configuration

* **Add a schedule slot**:
  ```bash
  arcane schedule add --category "Rust Programming" --time "09:00" --days "mon,tue,wed"
  ```
  Weekdays can be single tokens or comma-separated days, "everyday", "weekdays", or "weekend".

* **List schedule slots**:
  ```bash
  arcane schedule list
  ```

* **Remove a schedule slot**:
  ```bash
  arcane schedule remove --id 1
  ```

### 4. Temporary Overrides

To temporarily schedule or override a slot for today only (e.g., replacement or rest slot):
```bash
arcane today --time "14:00" --category "Rust Programming"
```
Or insert a rest block by using `rest` as the category:
```bash
arcane today --time "14:00" --category "rest"
```

### 5. Running the Active Timer Engine

To sequentially execute today's resolved timeline queue:
```bash
arcane start
```
* The engine catches up with mid-day schedules automatically.
* It fires desktop notification alerts at the start and completion of each block.
* Control options during the timer: `Space` (Pause/Resume), `S` (Skip current block), `Q` (Abort schedule run).
* Prompts you to score your recall rating (0-5) and write notes on completion to persist session logs.

### 6. Declarative Manifest Import and Export

* **Import schedule configuration**:
  ```bash
  arcane import path/to/manifest.toml
  ```
* **Export active database template**:
  ```bash
  arcane export > backup.toml
  ```

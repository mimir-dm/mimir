CREATE TABLE catalog_vehicles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    vehicle_type TEXT,
    size TEXT,
    cap_crew INTEGER,
    cap_passenger INTEGER,
    pace INTEGER,
    speed_text TEXT, -- Formatted speed string for display
    terrain_text TEXT, -- Comma-separated terrain types
    source TEXT NOT NULL,
    page INTEGER,
    full_vehicle_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Create indexes for common search patterns
CREATE INDEX idx_catalog_vehicles_name ON catalog_vehicles(name);
CREATE INDEX idx_catalog_vehicles_vehicle_type ON catalog_vehicles(vehicle_type);
CREATE INDEX idx_catalog_vehicles_size ON catalog_vehicles(size);
CREATE INDEX idx_catalog_vehicles_source ON catalog_vehicles(source);
CREATE INDEX idx_catalog_vehicles_terrain ON catalog_vehicles(terrain_text);
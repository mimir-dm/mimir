-- Add customization fields to module_monsters table
-- display_name: Optional custom name for the monster (e.g., "Frost Wight" using goblin stats)
-- notes: DM notes about customizations or thematic changes
ALTER TABLE module_monsters ADD COLUMN display_name TEXT;
ALTER TABLE module_monsters ADD COLUMN notes TEXT;

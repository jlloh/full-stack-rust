-- This file should undo anything in `up.sql`
DROP INDEX idx_assigned_number;
 
DROP INDEX idx_is_selected;

DROP INDEX idx_is_processed;

DROP TABLE queue;
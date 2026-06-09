-- Add migration script here
CREATE TABLE IF NOT EXISTS products (                                              
id TEXT PRIMARY KEY,                                                           
name TEXT NOT NULL,                                                            
slug TEXT NOT NULL,                                                            
description TEXT,                                                              
price REAL NOT NULL,                                                           
category TEXT NOT NULL,                                                        
sku TEXT NOT NULL,                                                             
created_at INTEGER NOT NULL                                                
);                                                                                                                                                            
CREATE INDEX IF NOT EXISTS idx_products_slug ON products(slug);                
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);        
CREATE INDEX IF NOT EXISTS idx_products_price ON products(price);              
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at);

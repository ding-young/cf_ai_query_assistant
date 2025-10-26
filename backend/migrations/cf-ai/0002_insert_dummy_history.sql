-- migration 0002: Insert dummy data into history table

INSERT INTO history (natural, sql, executed) VALUES
(
    'Find all users',
    'SELECT * FROM users;',
    1  -- Assumes this query was successfully 'executed' by the user.
),
(
    'How many users signed up in the last 3 days?',
    'SELECT COUNT(*) FROM users WHERE created_at >= date(''now'', ''-3 day'');',
    1  -- This one was also successfully run.
),
(
    'Show me the 5 most recent orders',
    'SELECT * FROM orders ORDER BY order_date DESC LIMIT 5;',
    0  -- This query was only generated, not yet executed.
),
(
    'Get the profile image for user 123',
    'SELECT prodile_img FROM users WHER id = 123;',
    0  -- Generated, but has a typo (prodile_img, WHER) and was not run (or failed).
),
(
    'Update user 456 to be an admin',
    'UPDATE users SET is_admin = 1 WHERE id = 456;',
    0  -- Generated, but not executed (running UPDATEs can be risky).
),
(
    'Find the total revenue for each product category',
    'SELECT category, SUM(price) AS total_revenue
     FROM products
     JOIN order_items ON products.id = order_items.product_id
     GROUP BY category;',
    1  -- A complex but successful query.
),
(
    'Add a new product called "Test Item"',
    'INSERT INTO products (name, price) VALUES (''Test Item'', 99.99);',
    0  -- Generated, not run.
);
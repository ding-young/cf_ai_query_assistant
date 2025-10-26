INSERT INTO users (id, name, email, is_admin, profile_img, created_at) VALUES
(
    123,
    'Alice',
    'alice@example.com',
    0,
    'alice.png',
    date('now', '-5 day')
),
(
    456,
    'Bob',
    'bob@example.com',
    0,
    'bob.png',
    date('now', '-2 day')
),
(
    789,
    'Charlie (Admin)',
    'charlie@example.com',
    1,
    'charlie.png',
    date('now', '-10 day')
);

INSERT INTO products (id, name, category, price) VALUES
(1, 'Laptop', 'Electronics', 1200.00),
(2, 'Coffee Maker', 'Appliances', 80.00),
(3, 'Wireless Mouse', 'Electronics', 45.50),
(4, 'T-Shirt', 'Apparel', 20.00),
(5, 'Blender', 'Appliances', 110.00);

INSERT INTO orders (id, user_id, order_date) VALUES
(1001, 123, date('now', '-1 day')), 
(1002, 456, date('now', '-2 day')),
(1003, 123, date('now', '-4 day')),
(1004, 789, date('now', '-5 day')),
(1005, 456, date('now', '-6 day')), 
(1006, 123, date('now', '-10 day'));

INSERT INTO order_items (order_id, product_id, quantity) VALUES
(1001, 1, 1),  -- Laptop (Electronics)
(1001, 3, 1),  -- Wireless Mouse (Electronics)
(1002, 2, 1),  -- Coffee Maker (Appliances)
(1003, 4, 5),  -- T-Shirt (Apparel)
(1004, 5, 1),  -- Blender (Appliances)
(1005, 3, 2);  -- Wireless Mouse (Electronics)
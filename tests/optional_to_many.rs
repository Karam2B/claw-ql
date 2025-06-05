// #[tokio::test]
// async fn group_by() {
//     let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
//
//     tracing_subscriber::fmt()
//         .with_max_level(tracing::Level::DEBUG)
//         .init();
//
//     let schema = Schema::default()
//         .infer_db::<Sqlite>()
//         .catch_errors_early()
//         .add_relation(Relation {
//             from: author,
//             to: book,
//         })
//         .add_collection(author)
//         .add_collection(book);
//
//     schema.migrate(&pool).await;
//
//     sqlx::query(
//         "
// INSERT INTO Author (name) VALUES
// ('Harper Lee'),
// ('George Orwell'),
// ('Jane Austen'),
// ('Stephen King'),
// ('J.K. Rowling');
//
// INSERT INTO Book (title) VALUES
// ('To Kill a Mockingbird'),
// ('1984'),
// ('Pride and Prejudice'),
// ('The Shining'),
// ('Harry Potter and the Sorcerer''s Stone'),
// ('Animal Farm'),
// ('It'),
// ('Go Set a Watchman'),
// ('The Stand'),
// ('The Casual Vacancy');
//
//
// INSERT INTO AuthorBook (author_id, book_id) VALUES
// (1, 1), -- Harper Lee wrote 'To Kill a Mockingbird'
// (2, 2), -- George Orwell wrote '1984'
// (3, 3), -- Jane Austen wrote 'Pride and Prejudice'
// (4, 4), -- Stephen King wrote 'The Shining'
// (5, 5), -- J.K. Rowling wrote 'Harry Potter and the Sorcerer''s Stone'
// (2, 6), -- George Orwell also wrote 'Animal Farm'
// (4, 7), -- Stephen King also wrote 'It'
// (1, 8), -- Harper Lee wrote 'Go Set a Watchman'
// (4, 9), -- Stephen King wrote 'The Stand'
// (5, 10); -- J.K. Rowling wrote 'The Casual Vacancy'
//     ",
//     )
//     .execute(&pool)
//     .await
//     .unwrap();
//
//     let res = get_one(author).relation(book).exec_op(pool).await;
//
//     pretty_assertions::assert_eq!(
//         res,
//         Some(SelectOneOutput {
//             id: 1,
//             attr: Author {
//                 name: "Harper Lee".to_string()
//             },
//             links: (vec![
//                 SimpleOutput {
//                     id: 1,
//                     attr: Book {
//                         title: "To Kill a Mockingbird".to_string(),
//                     },
//                 },
//                 SimpleOutput {
//                     id: 8,
//                     attr: Book {
//                         title: "Go Set a Watchman".to_string(),
//                     },
//                 },
//             ],),
//         },)
//     )
// }

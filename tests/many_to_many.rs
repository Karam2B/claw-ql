use claw_ql::links::group_by::{CountResult, count};
use claw_ql::links::relation_many_to_many::ManyToMany;
use claw_ql::{
    collections::CollectionBasic,
    operations::{
        LinkData, Relation, SimpleOutput,
        select_one::{SelectOneOutput, get_one},
    },
    schema::DynamicClient,
};
use claw_ql_macros::Collection;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool};

#[derive(Collection, Debug, PartialEq, Serialize, Deserialize)]
pub struct Student {
    pub name: String,
}

#[derive(Collection, Debug, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub code: String,
}

impl LinkData<course> for Relation<course, student> {
    type Spec = ManyToMany<course, student>;

    fn spec(self, table_1: course) -> Self::Spec
    where
        Self: Sized,
    {
        let junction = format!(
            "{table_1}{table_2}",
            table_1 = table_1.table_name(),
            table_2 = self.to.table_name()
        );
        ManyToMany {
            junction,
            id_1: format!("{}_id", table_1.table_name()),
            table_1,
            id_2: format!("{}_id", self.to.table_name()),
            table_2: self.to,
        }
    }
}
impl LinkData<student> for Relation<student, course> {
    type Spec = ManyToMany<student, course>;

    fn spec(self, table_1: student) -> Self::Spec
    where
        Self: Sized,
    {
        let junction = format!(
            "{author}{book}",
            author = table_1.table_name(),
            book = self.to.table_name()
        );
        ManyToMany {
            junction,
            id_1: format!("{}_id", table_1.table_name()),
            table_1,
            id_2: format!("{}_id", self.to.table_name()),
            table_2: self.to,
        }
    }
}

#[tokio::test]
async fn group_by() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let schema = DynamicClient::default()
        .infer_db::<Sqlite>()
        .catch_errors_early()
        .add_relation(Relation {
            from: student,
            to: course,
        })
        .add_collection(student)
        .add_collection(course);

    schema.migrate(&pool).await;

    sqlx::query(
        "
-- Insert some students
INSERT INTO Student (name) VALUES
('Alice Smith'),
('Bob Johnson'),
('Charlie Brown');

-- Insert some courses
INSERT INTO Course (code) VALUES
('CS101'),
('DB200'),
('WEB300');

-- Enroll students in courses (populate the junction table)
INSERT INTO StudentCourse (student_id, course_id) VALUES
(1, 1), -- Alice in CS101
(1, 2), -- Alice in DB200
(2, 1), -- Bob in CS101
(3, 2), -- Charlie in DB200
(3, 3); -- Charlie in WEB300
    ",
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = get_one(student)
        .relation(course)
        .exec_op(pool.clone())
        .await;

    pretty_assertions::assert_eq!(
        res,
        Some(SelectOneOutput {
            id: 1,
            attr: Student {
                name: "Alice Smith".to_string()
            },
            links: (vec![
                SimpleOutput {
                    id: 1,
                    attr: Course {
                        code: "CS101".to_string(),
                    },
                },
                SimpleOutput {
                    id: 2,
                    attr: Course {
                        code: "DB200".to_string(),
                    },
                },
            ],),
        },)
    );

    let res = get_one(course).link(count(student)).exec_op(pool.clone()).await;

    pretty_assertions::assert_eq!(
        res,
        Some(SelectOneOutput {
            id: 1,
            attr: Course {
                code: "CS101".to_string()
            },
            links: (
                CountResult(2), // there are two students enroled in CS101
            ),
        },)
    );

    let schema = DynamicClient::default()
        .infer_db::<Sqlite>()
        .to_build_json_client()
        // .add_link(count::dynamic_link())
        // .add_relation(Relation {
        //     from: student,
        //     to: course,
        // })
        .add_collection(student)
        .add_collection(course)
        .finish(pool);
}

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

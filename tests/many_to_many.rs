use claw_ql::collections::CollectionBasic;
use claw_ql::links::LinkData;
use claw_ql::links::group_by::{CountResult, count};
use claw_ql::links::relation::Relation;
use claw_ql::links::relation_many_to_many::ManyToMany;
use claw_ql::operations::select_one_op::select_one;
use claw_ql::operations::{CollectionOutput, LinkedOutput};
use claw_ql_macros::Collection;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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

// #[tokio::test]
async fn _group_by() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // let schema = BuilderPattern::default()
    //     .build_mode(to_migrate(Sqlite))
    //     .add_collection(student)
    //     .add_collection(course)
    //     .add_link(Relation {
    //         from: student,
    //         to: course,
    //     })
    //     .finish();
    //
    // schema.0.migrate(pool.clone()).await;

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

    let res = select_one(student)
        .relation(course)
        .exec_op(pool.clone())
        .await;

    pretty_assertions::assert_eq!(
        res,
        Some(LinkedOutput {
            id: 1,
            attr: Student {
                name: "Alice Smith".to_string()
            },
            links: (vec![
                CollectionOutput {
                    id: 1,
                    attr: Course {
                        code: "CS101".to_string(),
                    },
                },
                CollectionOutput {
                    id: 2,
                    attr: Course {
                        code: "DB200".to_string(),
                    },
                },
            ],),
        },)
    );

    let res = select_one(course)
        .link(count(student))
        .exec_op(pool.clone())
        .await;

    pretty_assertions::assert_eq!(
        res,
        Some(LinkedOutput {
            id: 1,
            attr: Course {
                code: "CS101".to_string()
            },
            links: (
                CountResult(2), // there are two students enroled in CS101
            ),
        },)
    );
}

macro_rules! migrator {
    ($($migration:ident),* $(,)?) => {
        pub struct Migrator;

        $(
            mod $migration {
                use sea_orm_migration::prelude::*;

                pub struct Migration;

                impl MigrationName for Migration {
                    fn name(&self) -> &str {
                        stringify!($migration)
                    }
                }

                #[async_trait::async_trait]
                impl MigrationTrait for Migration {
                    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                        manager
                            .get_connection()
                            .execute_unprepared(include_str!(
                                concat!("../sql/up/", stringify!($migration), ".sql")
                            ))
                            .await?;

                        Ok(())
                    }

                    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                        manager
                            .get_connection()
                            .execute_unprepared(include_str!(
                                concat!("../sql/down/d", stringify!($migration), ".sql")
                            ))
                            .await?;

                        Ok(())
                    }
                }
            }
        )*

        #[async_trait::async_trait]
        impl MigratorTrait for Migrator {
            fn migrations() -> Vec<Box<dyn MigrationTrait>> {
                vec![
                    $(
                        Box::new($migration::Migration),
                    )*
                ]
            }
        }
    };
}

pub(crate) use migrator;
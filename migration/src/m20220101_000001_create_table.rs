use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .col(
                        ColumnDef::new(Subscriptions::Id)
                            .uuid()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::Email)
                            .unique_key()
                            .not_null()
                            .text(),
                    )
                    .col(ColumnDef::new(Subscriptions::Name).not_null().text())
                    .col(
                        ColumnDef::new(Subscriptions::SubscribedAt)
                            .not_null()
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Subscriptions {
    Table,
    Id,
    Email,
    Name,
    SubscribedAt,
}

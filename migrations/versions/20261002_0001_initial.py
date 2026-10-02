"""Schéma initial versionné.

Revision ID: 20261002_0001
Revises:
Create Date: 2026-10-02
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "20261002_0001"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "users",
        sa.Column("id", sa.Integer(), nullable=False),
        sa.Column("email", sa.String(), nullable=False),
        sa.Column("password_hash", sa.String(), nullable=False),
        sa.Column("username", sa.String(), nullable=True),
        sa.Column("ade_ics_url", sa.String(), nullable=True),
        sa.Column("gemini_api_key", sa.String(), nullable=True),
        sa.Column("created_at", sa.DateTime(), nullable=True),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_users_email"), "users", ["email"], unique=True)
    op.create_index(op.f("ix_users_id"), "users", ["id"], unique=False)

    op.create_table(
        "config",
        sa.Column("id", sa.Integer(), nullable=False),
        sa.Column("user_id", sa.Integer(), nullable=True),
        sa.Column("username", sa.String(), nullable=True),
        sa.Column("ade_url", sa.String(), nullable=True),
        sa.Column("gemini_key", sa.String(), nullable=True),
        sa.ForeignKeyConstraint(["user_id"], ["users.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_config_user_id"), "config", ["user_id"], unique=False)

    op.create_table(
        "devoirs",
        sa.Column("id", sa.Integer(), nullable=False),
        sa.Column("user_id", sa.Integer(), nullable=True),
        sa.Column("titre", sa.String(), nullable=False),
        sa.Column("matiere", sa.String(), nullable=True),
        sa.Column("echeance", sa.String(), nullable=True),
        sa.Column("type", sa.String(), nullable=True),
        sa.Column("statut", sa.String(), nullable=True),
        sa.ForeignKeyConstraint(["user_id"], ["users.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(op.f("ix_devoirs_id"), "devoirs", ["id"], unique=False)
    op.create_index(op.f("ix_devoirs_user_id"), "devoirs", ["user_id"], unique=False)


def downgrade() -> None:
    op.drop_index(op.f("ix_devoirs_user_id"), table_name="devoirs")
    op.drop_index(op.f("ix_devoirs_id"), table_name="devoirs")
    op.drop_table("devoirs")
    op.drop_index(op.f("ix_config_user_id"), table_name="config")
    op.drop_table("config")
    op.drop_index(op.f("ix_users_id"), table_name="users")
    op.drop_index(op.f("ix_users_email"), table_name="users")
    op.drop_table("users")

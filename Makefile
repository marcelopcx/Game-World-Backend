.PHONY: setup reset-db run check db-logs sqlx-prepare

# Regenera .sqlx/ contra Postgres local (requiere Docker + esquema aplicado).
sqlx-prepare:
	@DATABASE_URL="postgres://gameworld:secret123@127.0.0.1:5432/game_world?options=-csearch_path%3Dgame_world" \
		cargo sqlx prepare

setup:
	@./scripts/setup.sh

reset-db:
	@./scripts/reset-db.sh

run:
	@cargo run

check:
	@cargo check

db-logs:
	@docker compose logs -f db

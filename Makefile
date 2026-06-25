.PHONY: setup reset-db migrate-db run check db-logs sqlx-prepare test-api bruno

# Regenera .sqlx/ contra Postgres local (requiere Docker + esquema aplicado).
sqlx-prepare:
	@DATABASE_URL="postgres://gameworld:secret123@127.0.0.1:5432/game_world?options=-csearch_path%3Dgame_world" \
		cargo sqlx prepare

setup:
	@./scripts/setup.sh

migrate-db:
	@./scripts/migrate-db.sh

reset-db:
	@./scripts/reset-db.sh

run:
	@cargo run

check:
	@cargo check

db-logs:
	@docker compose logs -f db

test-api:
	@./scripts/test-api.sh

bruno:
	@chmod +x scripts/run-bruno.sh
	@mkdir -p API/reports
	@./scripts/run-bruno.sh

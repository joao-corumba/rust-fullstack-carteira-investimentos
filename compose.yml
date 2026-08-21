services:
  db:
    image: postgres:18.1-alpine3.23
    environment:
     POSTGRES_USER: postgres
     POSTGRES_PASSWORD: postgres
     POSTGRES_DATABASE: postgres
    ports:
      - 5432:5432
    volumes:
       - postgres:/var/lib/postgresql/data

volumes:
  postgres:

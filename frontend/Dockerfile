FROM node:24-alpine AS base

WORKDIR /app

ENV COREPACK_HOME=/corepack

RUN corepack enable

FROM base AS deps

WORKDIR /app/frontend

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./

RUN --mount=type=cache,target=/root/.local/share/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile

FROM base AS builder

WORKDIR /app/frontend

COPY --from=deps /app/frontend/node_modules ./node_modules

COPY frontend /app/frontend
COPY proto /app/proto

RUN pnpm run proto:generate
RUN pnpm build

FROM base AS prod-deps

WORKDIR /app/frontend

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./

RUN --mount=type=cache,target=/root/.local/share/pnpm/store,sharing=locked \
    pnpm install --prod --frozen-lockfile --ignore-scripts

FROM node:24-alpine AS runtime

WORKDIR /app/frontend

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000

COPY --from=builder /app/frontend/build ./build
COPY --from=builder /app/frontend/package.json ./package.json
COPY --from=prod-deps /app/frontend/node_modules ./node_modules

USER node

EXPOSE 3000

CMD ["node", "build"]

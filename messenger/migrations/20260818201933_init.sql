-- Таблица чатов (комнат)
CREATE TABLE IF NOT EXISTS chats (
    id UUID PRIMARY KEY,
    order_id UUID, -- Опционально: привязка к заказу на ремонт
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Таблица участников чата
CREATE TABLE IF NOT EXISTS chat_members (
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    user_id UUID NOT NULL, -- ID пользователя (из Keycloak)
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chat_id, user_id)
);

-- Таблица сообщений
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY,
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индекс для быстрой загрузки истории чата по времени
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages(chat_id, created_at DESC);

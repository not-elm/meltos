const BASE_URI = "https://room.meltos.net";

class HttpRoomClient {
    constructor(
        configs
    ) {
        this.configs = configs;
    }

    get roomId() {
        return this.configs.room_id;
    }

    get sessionId() {
        return this.configs.session_id;
    }

    static async open() {
        const opened = await httpOpen();
        return new HttpRoomClient(opened);
    }

    sync = async () => {
        const response = await fetch(this.apiUri(), {
            ...headers(this.sessionId),
        });
        const json = await response.json();
        return json;
    };

    create = async (title) => {
        return await httpCreate(this.configs.room_id, this.configs.session_id, title);
    };

    speak = async (discussionId, message) => {
        return await httpSpeak(
            this.configs.room_id,
            this.configs.session_id,
            discussionId,
            message
        );
    };

    reply = async (
        discussionId,
        to,
        message
    ) => {
        return await httpReply(
            this.configs.room_id,
            this.configs.session_id,
            discussionId,
            to,
            message
        );
    };

    leave = async () => {
        await fetch(`${BASE_URI}/room/${this.roomId}`, {
            method: "DELETE",
            ...headers(this.sessionId),
        });
    };

    apiUri = (uri) => {
        return !!uri
            ? `${BASE_URI}/room/${this.roomId}/${uri}`
            : `${BASE_URI}/room/${this.roomId}`;
    };
}

const httpOpen = async (body) => {
    const response = await fetch(`${BASE_URI}/room/open`, {
        method: "POST",
        headers: {
            "content-type": "application/json",
        },
        body,
    });
    const json = await response.text();
    return json;
};

const httpCreate = async (
    roomId,
    sessionId,
    title
) => {
    const response = await fetch(
        `${BASE_URI}/room/${roomId}/discussion/global/create`,
        {
            method: "POST",
            ...headers(sessionId),
            body: JSON.stringify({
                title,
            }),
        }
    );
    const json = await response.json();
    return json;
};

const httpSpeak = async (
    roomId,
    sessionId,
    discussionIdg,
    text
) => {
    const response = await fetch(
        `${BASE_URI}/room/${roomId}/discussion/global/speak`,
        {
            method: "POST",
            ...headers(sessionId),
            body: JSON.stringify({
                discussion_id: discussionId,
                text,
            }),
        }
    );
    const json = await response.json();
    return json;
};

const httpReply = async (
    roomId,
    sessionId,
    discussionId,
    to,
    message
) => {
    const response = await fetch(
        `${BASE_URI}/room/${roomId}/discussion/global/reply`,
        {
            method: "POST",
            ...headers(sessionId),
            body: JSON.stringify({
                discussion_id: discussionId,
                to,
                text: message,
            }),
        }
    );
    const json = await response.json();
    return json;
};

const headers = (sessionId) => {
    return {
        headers: {
            "content-type": "application/json",
            "set-cookie": `session_id=${sessionId}`,
        },
    };
};


module.exports = {
    open: async (body) => {
        const opened = await httpOpen(body);
        return opened;
    }
}
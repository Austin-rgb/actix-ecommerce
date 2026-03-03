import requests

sess = requests.Session()
URL = "http://localhost:8080"
COOKIES = {}


def register_user(username, password, url="auth"):
    data = {"username": username, "password": password}
    response = sess.post(f"{URL}/{url}/auth/register", json=data)
    return response


def login_user(username, password, url="auth"):
    data = {"username": username, "password": password}
    response = sess.post(f"{URL}/{url}/auth/login", json=data)
    token = response.cookies.get("access_token")
    sess.headers["Authorization"] = f"Bearer {token}"
    return response


def create_conversation(participants, url="messages"):
    data = {"participants": participants}
    response = sess.post(f"{URL}/{url}/conversations", json=data)
    return response


def get_messages(url="/messages/inbox/messages"):
    response = sess.get(f"{URL}/{url}", cookies=COOKIES)
    return response


if __name__ == "__main__":
    # Example usage
    reg_response = register_user("testuser", "testpass")
    print("Register Response:", reg_response.status_code, reg_response.json())

    login_response = login_user("testuser", "testpass")
    print("Login Response:", login_response.status_code, login_response.json())

    # conv_response = create_conversation(["testuser", "anotheruser"])
    # print(
    #     "Create Conversation Response:",
    #     conv_response.status_code,
    #     conv_response.content,
    # )
    messages_response = get_messages()
    print(
        "Get messages response",
        messages_response.status_code,
        messages_response.content,
    )

import socket
import threading

HOST = '0.0.0.0'
PORT = 8081

def handle_client(client_socket):
    while True:
        try:
            message = client_socket.recv(1024).decode('utf-8')
            if not message:
                break
            print(f"Received from client: {message}")
        except ConnectionResetError:
            break

    client_socket.close()
    print("Client disconnected")

def send_to_client(client_socket):
    while True:
        try:
            message = input()
            client_socket.send(message.encode('utf-8'))
        except ConnectionResetError:
            break

def main():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind((HOST, PORT))
    server.listen(5)
    print(f"[*] Listening on {HOST}:{PORT}")

    while True:
        client_socket, addr = server.accept()
        print(f"[*] Accepted connection from {addr}")

        client_handler = threading.Thread(target=handle_client, args=(client_socket,))
        client_handler.start()

        input_handler = threading.Thread(target=send_to_client, args=(client_socket,))
        input_handler.start()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n[*] Exiting...")
        exit(0)

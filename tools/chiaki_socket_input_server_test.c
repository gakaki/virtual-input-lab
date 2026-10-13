/**
 * @file tools/chiaki_socket_input_server_test.c
 * @brief Reference receiver for the chiaki-ng socket input protocol.
 *
 * Binds a Unix datagram socket, receives controller state messages, validates
 * them with the very same header the chiaki-ng patch uses, and prints the
 * decoded state. Used to verify the Python and Rust clients without building
 * chiaki-ng.
 *
 *   clang -std=c11 -Wall -Wextra -I patches \
 *     tools/chiaki_socket_input_server_test.c -o /tmp/chiaki_server_test
 *   /tmp/chiaki_server_test /tmp/test-input.sock 5
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include "socketinput_protocol.h"

static void print_state(const ChiakiSocketInputMessage *message, int index) {
  printf("message %d: buttons=0x%04x l2=%u r2=%u left=(%d,%d) right=(%d,%d)\n",
         index,
         message->buttons & 0xFFFFu,
         message->l2_state,
         message->r2_state,
         message->left_x,
         message->left_y,
         message->right_x,
         message->right_y);
}

int main(int argc, char **argv) {
  const char *path = argc > 1 ? argv[1] : CHIAKI_SOCKET_INPUT_DEFAULT_PATH;
  const int wanted = argc > 2 ? atoi(argv[2]) : 5;

  const int fd = socket(AF_UNIX, SOCK_DGRAM, 0);
  if (fd < 0) {
    perror("socket");
    return 2;
  }

  struct sockaddr_un address;
  memset(&address, 0, sizeof(address));
  address.sun_family = AF_UNIX;
  if (strlen(path) >= sizeof(address.sun_path)) {
    fprintf(stderr, "socket path too long: %s\n", path);
    return 2;
  }
  strcpy(address.sun_path, path);

  unlink(path);
  if (bind(fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
    perror("bind");
    return 2;
  }

  struct timeval timeout = {.tv_sec = 5, .tv_usec = 0};
  setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));

  printf("listening on %s (want %s)\n", path, wanted > 0 ? "N messages" : "messages until timeout");
  fflush(stdout);

  int received = 0;
  int rejected = 0;
  while (wanted <= 0 || received < wanted) {
    ChiakiSocketInputMessage message;
    const ssize_t count = recv(fd, &message, sizeof(message), 0);
    if (count < 0) {
      if (errno == EAGAIN || errno == EWOULDBLOCK) {
        break;
      }
      if (errno == EINTR) {
        continue;
      }
      perror("recv");
      break;
    }
    if (!chiaki_socket_input_validate(&message, (unsigned)count)) {
      rejected++;
      printf("message rejected (%ld bytes, magic=0x%08x version=%u size=%u)\n",
             (long)count,
             message.magic,
             message.version,
             message.size);
      continue;
    }
    print_state(&message, received + 1);
    received++;
  }

  close(fd);
  unlink(path);
  printf("received=%d rejected=%d\n", received, rejected);
  return received > 0 ? 0 : 1;
}

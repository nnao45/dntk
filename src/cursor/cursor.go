package cursor

// https://www.linuxquestions.org/questions/programming-9/get-cursor-position-in-c-947833/

/*
#cgo LDFLAGS: -L/usr/lib
#include <unistd.h>
#include <fcntl.h>
#include <termios.h>
#include <errno.h>

#define   RD_EOF   -1
#define   RD_EIO   -2

static inline int rd(const int fd)
{
    unsigned char   buffer[4];
    ssize_t         n;

    while (1) {

        n = read(fd, buffer, 1);
        if (n > (ssize_t)0)
            return buffer[0];

        else
        if (n == (ssize_t)0)
            return RD_EOF;

        else
        if (n != (ssize_t)-1)
            return RD_EIO;

        else
        if (errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK)
            return RD_EIO;
    }
}

static inline int wr(const int fd, const char *const data, const size_t bytes)
{
    const char       *head = data;
    const char *const tail = data + bytes;
    ssize_t           n;

    while (head < tail) {

        n = write(fd, head, (size_t)(tail - head));
        if (n > (ssize_t)0)
            head += n;

        else
        if (n != (ssize_t)-1)
            return EIO;

        else
        if (errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK)
            return errno;
    }

    return 0;
}

int current_tty(void)
{
    const char *dev;
    int         fd;

    dev = ttyname(STDIN_FILENO);
    if (!dev)
        dev = ttyname(STDOUT_FILENO);
    if (!dev)
        dev = ttyname(STDERR_FILENO);
    if (!dev) {
        errno = ENOTTY;
        return -1;
    }

    do {
        fd = open(dev, O_RDWR | O_NOCTTY);
    } while (fd == -1 && errno == EINTR);
    if (fd == -1)
        return -1;

    return fd;
}


int cursor_position(const int tty, int *const rowptr, int *const colptr)
{
    struct termios  saved, temporary;
    int             retval, result, rows, cols, saved_errno;

    if (tty == -1)
        return ENOTTY;

    saved_errno = errno;

    do {
        result = tcgetattr(tty, &saved);
    } while (result == -1 && errno == EINTR);
    if (result == -1) {
        retval = errno;
        errno = saved_errno;
        return retval;
    }

    do {
        result = tcgetattr(tty, &temporary);
    } while (result == -1 && errno == EINTR);
    if (result == -1) {
        retval = errno;
        errno = saved_errno;
        return retval;
    }

    temporary.c_lflag &= ~ICANON;
    temporary.c_lflag &= ~ECHO;
    temporary.c_cflag &= ~CREAD;

    do {

        do {
            result = tcsetattr(tty, TCSANOW, &temporary);
        } while (result == -1 && errno == EINTR);
        if (result == -1) {
            retval = errno;
            break;
        }

        retval = wr(tty, "\033[6n", 4);
        if (retval)
            break;

        retval = EIO;

        result = rd(tty);
        if (result != 27)
            break;

        result = rd(tty);
        if (result != '[')
            break;

        rows = 0;
        result = rd(tty);
        while (result >= '0' && result <= '9') {
            rows = 10 * rows + result - '0';
            result = rd(tty);
        }

        if (result != ';')
            break;

        cols = 0;
        result = rd(tty);
        while (result >= '0' && result <= '9') {
            cols = 10 * cols + result - '0';
            result = rd(tty);
        }

        if (result != 'R')
            break;

        if (rowptr)
            *rowptr = rows;

        if (colptr)
            *colptr = cols;

        retval = 0;

    } while (0);

    do {
        result = tcsetattr(tty, TCSANOW, &saved);
    } while (result == -1 && errno == EINTR);
    if (result == -1 && !retval)
        retval = errno;

    return retval;
}

int get_pos(void)
{
    int         fd, row, col;
    char        buffer[64];
    char *const tail = buffer + sizeof(buffer);
    char       *head = buffer + sizeof(buffer);

    fd = current_tty();
    if (fd == -1)
        return -1;

    row = 0;
    col = 0;
    if (cursor_position(fd, &row, &col))
        return -2;

    if (row < 1 || col < 1)
        return -3;

    {   int    u = row;
        do {
            *(--head) = '0' + (u % 10U);
            u /= 10U;
        } while (u);
    }

    return row;
}

*/
import "C"

type cursor struct {
	//Col C.int
	Row C.int
}

func GetCursorRow() C.int {
	cur := &cursor{
		Row: C.get_pos(),
	}
	return cur.Row
}

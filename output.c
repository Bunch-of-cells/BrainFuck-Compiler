#include <stdio.h>
#include <termios.h>

int getch() {
    struct termios old, new;
    int ch;
    tcgetattr(0, &old);
    new = old;
    new.c_lflag &= ~ICANON;
    new.c_lflag &= ~ECHO;
    tcsetattr(0, TCSANOW, &new);
    ch = getchar();
    tcsetattr(0, TCSANOW, &old);
    return ch;
}

int main() {
    char mem[30000];
    char* ptr = mem + 0;

	*ptr += 5;
	while (*ptr) {
	}
	return 0;
}

#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

void print_int(int x) {
	printf("%d\n", x);
}

void print_bool(bool x) {
	if (x) {
		puts("true");
	} else {
		puts("false");
	}
}

void print_str(char *str) {
	puts(str);
}

bool str_eq(char *lhs, char *rhs) {
	int result = strcmp(lhs, rhs);
	return result == 0;
}

int str_len(char *str) {
	return strlen(str);
}

// TODO: this function causes memory leaks which is probably an issue for JIT
char* str_concat(char *lhs, char *rhs) {
	// is this correct??? (I hate C)
	char *new_str = malloc(str_len(lhs) + str_len(rhs) + 1);
	strcpy(new_str, lhs);
	strcat(new_str, rhs);
	return new_str;
}

void assert_int_eq(int lhs, int rhs) {
	if (lhs != rhs) {
		printf("assertion failed, %d != %d\n", lhs, rhs);
		exit(1);
	}
}

void assert_bool_eq(bool lhs, bool rhs) {
	if (lhs != rhs) {
		printf("assertion failed, %d != %d\n", lhs, rhs);
		exit(1);
	}
}

void assert_str_eq(char *lhs, char *rhs) {
	if (!str_eq(lhs, rhs)) {
		printf("assertion failed, %s != %s\n", lhs, rhs);
		exit(1);
	}
}

void panic() {
	puts("panic");
	exit(1);
}

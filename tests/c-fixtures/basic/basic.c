#include "basic.h"

int add(int a, int b) { return a + b; }

int multiply(int a, int b) { return a * b; }

int point_distance(struct Point p) {
    int x = p.x < 0 ? -p.x : p.x;
    int y = p.y < 0 ? -p.y : p.y;
    return x + y;
}

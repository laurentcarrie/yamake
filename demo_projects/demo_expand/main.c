#include "project_1/wrapper.h"
#include <stdio.h>
#include <math.h>

// this file is generated and does not belong to the project sources
#include "demo_cluster/languages.h"

int main() {

    for i = 0; i < LANGUAGES_COUNT; i++ {
        printf("language %d is %s\n", i, get_language_name(i));
        printf("language %d is %s\n", i, get_language_hello(i));
    }

}

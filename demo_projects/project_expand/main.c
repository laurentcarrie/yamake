#include "project_expand/wrapper.h"
#include <stdio.h>
#include <math.h>
#include "project_expand/generated/languages.h"

int main() {
    for (int i=0;i<N_languages;i++) {
        printf("%s\n", languages[i]()) ;
    }

}

build:
    gcc src/main.c -o dist/main

run: build
    ./dist/main

clean:
    rm -rf dist
    mkdir dist

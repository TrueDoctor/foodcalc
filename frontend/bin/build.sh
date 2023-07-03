#!/bin/sh

DIST=../backend/static

report() {
  touch tmp/build.log
  ERRORS=`cat tmp/build.log`
  if [ -n "$ERRORS" ]; then
    echo "Comiled with errors"
    # to also print errors in console we just compile a second time
    elm make src/Main.elm
    VALUE=`date -r tmp/build.log`
    printf "refresh('" > $DIST/tmp/timestamp.js
    printf "$VALUE" >>  $DIST/tmp/timestamp.js
    printf "', " >>  $DIST/tmp/timestamp.js
    cat tmp/build.log >> $DIST/tmp/timestamp.js
    printf ");" >> $DIST/tmp/timestamp.js
  else
    echo "Compiled without errors"
    VALUE=`date -r elm.js`
    TIMESTAMP_JS_TEMPLATE="refresh('${VALUE}')"
    INTERPOLATED=`echo "${TIMESTAMP_JS_TEMPLATE}" | sed "s/VALUE/${VALUE}/" | sed "s/ERROR//" `
    echo "$INTERPOLATED" > $DIST/tmp/timestamp.js
  fi
}


buildCode() {
  echo "Compiling ⚔️"
  elm make src/Main.elm --output=../backend/static/elm.js --report=json 2> tmp/build.log
  report
}


while inotifywait -qqre modify "src"; do
  buildCode
done

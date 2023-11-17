module Settings exposing (..)





backend : String -> String
backend path =
    "http://localhost:3000" ++ path
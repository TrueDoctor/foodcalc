module Test.StringUtils exposing (..)


contains : String -> String -> Bool
contains a b =
    String.contains (String.toLower a) (String.toLower b)


fuzzyContains : String -> String -> Bool
fuzzyContains a b =
    String.foldl
        (\c d ->
            if String.startsWith (String.fromChar c) d then
                String.dropLeft 1 d

            else
                d
        )
        b
        a
        == ""

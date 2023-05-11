module Settings exposing (..)

import Model exposing (Tab(..))

tabName : Tab -> String
tabName tab =
    case tab of
        Ingredients _ ->
            "Ingredients"

        Recipes ->
            "Recipes"

        Events ->
            "Events"


backend : String -> String
backend path =
    "http://localhost:3000" ++ path
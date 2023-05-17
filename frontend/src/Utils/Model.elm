module Utils.Model exposing (..)

import Http


type RemoteData a e
    = NotAsked
    | Loading
    | Success a
    | Failure e


type alias WebData a =
    RemoteData a Http.Error


type alias Unit =
    { unit_id : Int
    , name : String
    }


type alias DropdownData a =
    { list : List a
    , selected : a
    , filter : String
    , open : Bool
    }


newDropdownData : List a -> a -> DropdownData a
newDropdownData list selected =
    { list = list
    , selected = selected
    , filter = ""
    , open = False
    }


type alias DropdownEvents a msg =
    { onFilter : String -> msg
    , onSelect : a -> msg
    , property : a -> String
    }

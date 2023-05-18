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
    { selected : Maybe a
    , filter : String
    , open : Bool
    }


newDropdownData : Maybe a -> DropdownData a
newDropdownData selected =
    { selected = selected
    , filter = ""
    , open = False
    }


type alias DropdownEvents a msg =
    { onFilter : String -> msg
    , onSelect : a -> msg
    , property : a -> String
    }

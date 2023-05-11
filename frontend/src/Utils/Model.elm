module Utils.Model exposing (..)

import Http


type RemoteData a e
    = NotAsked
    | Loading
    | Success a
    | Failure e


type alias WebData a =
    RemoteData a Http.Error

module WebData exposing (..)
import Http



type RemoteData a e
    = NotAsked
    | Loading
    | Success a
    | Failure e


type alias WebData a =
    RemoteData a Http.Error



fromResult : Result Http.Error a -> WebData a
fromResult r =
    case r of
        Ok a ->
            Success a

        Err e ->
            Failure e


map : (a -> b) -> WebData a -> WebData b
map f2 wd =
    case wd of
        Success a ->
            Success (f2 a)

        Failure e ->
            Failure e

        NotAsked ->
            NotAsked

        Loading ->
            Loading


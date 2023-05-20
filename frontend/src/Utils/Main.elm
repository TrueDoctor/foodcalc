module Utils.Main exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Utils.Model exposing (..)


toWebdata : Result Http.Error a -> WebData a
toWebdata r =
    case r of
        Ok a ->
            Success a

        Err e ->
            Failure e


mapWebdata : (a -> b) -> WebData a -> WebData b
mapWebdata f wd =
    case wd of
        Success a ->
            Success (f a)

        Failure e ->
            Failure e

        NotAsked ->
            NotAsked

        Loading ->
            Loading


role : String -> Attribute msg
role =
    attribute "role"


nameFilter : String -> String -> Bool
nameFilter filter name =
    String.contains (String.toLower filter) (String.toLower name)


propertyFilter : (a -> String) -> String -> a -> Bool
propertyFilter property filter item =
    String.contains (String.toLower filter) (String.toLower (property item))

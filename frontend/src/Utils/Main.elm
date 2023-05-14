module Utils.Main exposing (..)

import Http 
import Utils.Model exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Html exposing (..)

mapWebdata : Result Http.Error a -> WebData a
mapWebdata r =
    case r of
        Ok a ->
            Success a

        Err e ->
            Failure (Debug.log "" e)

role : String -> Attribute msg
role = attribute "role"

nameFilter: String -> String -> Bool
nameFilter filter name =
    String.contains (String.toLower filter) (String.toLower name)
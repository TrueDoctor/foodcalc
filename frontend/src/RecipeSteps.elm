module RecipeSteps exposing (..)

import Element exposing (..)
import Element.Input
import FeatherIcons as FI
import Html.Attributes exposing (title)
import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Settings exposing (backend)
import WebData exposing (RemoteData(..), WebData)


type alias Step =
    { id : Maybe Int
    , title : String
    , order : String
    , description : String
    , duration : String
    , durationPerKg : String
    }


type StepMsg
    = SetTitle String
    | SetOrder String
    | SetDescription String
    | SetFixedDuration String
    | SetDurationPerKg String
    | NewStep
    | DeleteStep
    | StepsSaved (Result Http.Error Int)
    | StepsRecieved (Result Http.Error (List Step))


viewSteps : (Maybe Step -> StepMsg -> msg) -> WebData (List Step) -> Element msg
viewSteps mapMsg wd =
    case wd of
        Success steps ->
            column [ width fill, spacing 20, paddingXY 0 20 ]
                (List.map (viewStep mapMsg) steps
                    ++ [ Element.Input.button []
                            { onPress = Just <| mapMsg Nothing NewStep
                            , label = el [ paddingXY 30 10 ] (html (FI.toHtml [] FI.plus))
                            }
                       ]
                )

        Failure _ ->
            el [] (text "Failed to load Steps")

        _ ->
            el [] (text "Loading")


viewStep : (Maybe Step -> StepMsg -> msg) -> Step -> Element msg
viewStep mapMsg step =
    Element.map (mapMsg <| Just step) <|
        column [ width fill, spacing 10 ]
            [ row [ width fill, paddingXY 20 0, spacing 30 ]
                [ el [ width <| px 100 ] <|
                    Element.Input.text []
                        { label = Element.Input.labelAbove [] <| text "Order"
                        , onChange = SetOrder
                        , placeholder = Just <| Element.Input.placeholder [] (text "Order")
                        , text = step.order
                        }
                , el [ width fill ] <|
                    Element.Input.text []
                        { label = Element.Input.labelAbove [] <| text "Title"
                        , onChange = SetTitle
                        , placeholder = Just <| Element.Input.placeholder [] <| text "Title"
                        , text = step.title
                        }
                ]
            , el [ width fill, paddingXY 20 0 ] <|
                Element.Input.text []
                    { label = Element.Input.labelAbove [] <| text "Description"
                    , onChange = SetDescription
                    , placeholder = Just <| Element.Input.placeholder [] <| text "Description"
                    , text = step.description
                    }
            , row [ width fill, paddingXY 20 0, spacing 30 ]
                [ el [ width <| fillPortion 1 ] <|
                    Element.Input.text []
                        { label = Element.Input.labelAbove [] <| text "Fixed duration"
                        , onChange = SetFixedDuration
                        , placeholder = Just <| Element.Input.placeholder [] (text "((h:)m:)s")
                        , text = step.duration
                        }
                , el [ width <| fillPortion 1 ] <|
                    Element.Input.text []
                        { label = Element.Input.labelAbove [] <| text "Duration per kg"
                        , onChange = SetDurationPerKg
                        , placeholder = Just <| Element.Input.placeholder [] <| text "Duration per kg"
                        , text = step.durationPerKg
                        }
                ]
            ]


updateSteps : StepMsg -> WebData (List Step) -> Maybe Step -> ( WebData (List Step), Cmd msg )
updateSteps msg wd maybeStep =
    let
        replaceIf f step s =
            if s == step then
                f s

            else
                s

        replace f step =
            List.map (replaceIf f step)
    in
    case ( msg, wd, maybeStep ) of
        ( NewStep, Success allSteps, _ ) ->
            ( Success <| allSteps ++ [ { id = Nothing, order = "", title = "", description = "", duration = "0", durationPerKg = "0" } ], Cmd.none )

        ( SetOrder order, Success allSteps, Just step ) ->
            ( Success <| replace (\s -> { s | order = order }) step allSteps, Cmd.none )

        ( SetTitle title, Success allSteps, Just step ) ->
            ( Success <| replace (\s -> { s | title = title }) step allSteps, Cmd.none )

        ( SetDurationPerKg dur, Success allSteps, Just step ) ->
            ( Success <| replace (\s -> { s | durationPerKg = dur }) step allSteps, Cmd.none )

        ( SetFixedDuration dur, Success allSteps, Just step ) ->
            ( Success <| replace (\s -> { s | duration = dur }) step allSteps, Cmd.none )

        ( SetDescription description, Success allSteps, Just step ) ->
            ( Success <| replace (\s -> { s | description = description }) step allSteps, Cmd.none )

        ( StepsRecieved result, _, _ ) ->
            case Debug.log "" result of
                Ok list ->
                    ( Success <| List.sortBy (Maybe.withDefault 0 << String.toFloat << .order) list, Cmd.none )

                Err e ->
                    ( Failure e, Cmd.none )

        _ ->
            ( wd, Cmd.none )



-- Encode


encodeSteps : Int -> WebData (List Step) -> Maybe Encode.Value
encodeSteps id steps =
    let
        mapValidStep a b =
            case ( encodeStep id a, b ) of
                ( Just i, Just list ) ->
                    Just (list ++ [ i ])

                _ ->
                    Nothing

        encoded =
            case steps of
                Success s ->
                    List.foldl mapValidStep (Just []) s

                _ ->
                    Nothing
    in
    encoded
        |> Maybe.map (Encode.list identity)


encodeDuration : Int -> Encode.Value
encodeDuration secs =
    Encode.object [ ( "secs", Encode.int secs ), ( "nanos", Encode.int 0 ) ]


parseDuration : String -> Maybe Encode.Value
parseDuration s =
    let
        parts =
            List.map (String.toInt << (++) "0") << String.split ":"

        timeParts =
            (List.map2 <| Maybe.map2 (*)) [ Just 3600, Just 60, Just 1 ] << parts

        time list =
            if List.any (Maybe.withDefault True << Maybe.map (always False)) list then
                Nothing

            else
                Just <| List.sum <| List.filterMap identity list
    in
    Maybe.map encodeDuration <| time <| timeParts s


decodeDuration : Decode.Decoder String
decodeDuration =
    let
        hours s =
            s // 3600

        minutes s =
            modBy 60 <| s // 60

        seconds =
            modBy 60

        time secs =
            String.join ":" (List.map (\t -> String.fromInt <| t secs) [ hours, minutes, seconds ])
    in
    Decode.map time <| Decode.field "secs" Decode.int


encodeStep : Int -> Step -> Maybe Encode.Value
encodeStep id step =
    Maybe.map3
        (\order duration durationPerKg ->
            Encode.object
                [ ( "step_id", Maybe.withDefault (Encode.int -1) <| Maybe.map Encode.int step.id )
                , ( "step_name", Encode.string step.title )
                , ( "step_description", Encode.string step.description )
                , ( "step_order", Encode.float order )
                , ( "recipe_id", Encode.int id )
                , ( "fixed_duration", duration )
                , ( "duration_per_kg", durationPerKg )
                ]
        )
        (String.toFloat step.order)
        (parseDuration step.duration)
        (parseDuration step.durationPerKg)



-- Decode


decodeStep : Decode.Decoder Step
decodeStep =
    Decode.map6 (\id name order desc dur durKg -> Step id name (String.fromFloat order) desc dur durKg)
        (Decode.field "step_id" <| Decode.maybe Decode.int)
        (Decode.field "step_name" <| Decode.string)
        (Decode.field "step_order" <| Decode.float)
        (Decode.field "step_description" <| Decode.string)
        (Decode.field "fixed_duration" decodeDuration)
        (Decode.field "duration_per_kg" decodeDuration)



-- Update


updateRecipeSteps : WebData (List Step) -> Int -> Cmd StepMsg
updateRecipeSteps steps id =
    case encodeSteps id steps of
        Just body ->
            Http.post
                { url = "http://localhost:3000/recipes/" ++ String.fromInt id ++ "/steps/update"
                , body = Http.jsonBody body
                , expect = Http.expectJson StepsSaved Decode.int
                }

        _ ->
            Cmd.none


fetchSteps : Int -> Cmd StepMsg
fetchSteps id =
    Http.get
        { url = backend <| "/recipes/" ++ String.fromInt id ++ "/steps/list"
        , expect = Http.expectJson StepsRecieved (Decode.list decodeStep)
        }

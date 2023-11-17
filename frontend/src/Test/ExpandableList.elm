module Test.ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg, update, view)

import Bitwise exposing (and)
import Element exposing (..)
import Element.Background
import Element.Events exposing (onClick)
import Element.Input
import FeatherIcons as FI
import Ingredients.Model exposing (Modal(..))


type alias ExpandableList a msg elementMsg =
    { search : String
    , filter : String -> a -> Bool
    , items : List ( Bool, a )
    , viewElement : Attribute msg -> Bool -> a -> Element msg
    , mapMsg : ExpandableListMsg a elementMsg -> msg
    , update : elementMsg -> a -> ( a, Cmd msg )
    , add : Maybe (() -> a)
    , expandItem : Maybe elementMsg
    }


type ExpandableListMsg a elementMsg
    = InputChanged String
    | ElementMsg a elementMsg
    | ElementExpand a Bool
    | AddElement


mapElementMsg : a -> elementMsg -> ExpandableListMsg a elementMsg
mapElementMsg =
    ElementMsg


viewFilter : String -> Element (ExpandableListMsg a elementMsg)
viewFilter search =
    el [ width fill ]
        (Element.Input.text [ centerX, width fill ]
            { onChange = InputChanged
            , text = search
            , label = Element.Input.labelAbove [] none
            , placeholder = Nothing
            }
        )


viewAdd : Maybe (() -> a) -> Element (ExpandableListMsg a elementMsg)
viewAdd add =
    case add of
        Just _ ->
            el [ padding 10 ]
                (Element.Input.button []
                    { onPress = Just AddElement
                    , label = el [ paddingXY 30 10 ] (html (FI.toHtml [] FI.plus))
                    }
                )

        Nothing ->
            none


view : ExpandableList a msg elementMsg -> Element msg
view { search, filter, items, viewElement, mapMsg, add } =
    let
        bg i =
            if and 1 i == 0 then
                rgb255 250 250 250

            else
                rgb255 230 230 230

        expand item expanded =
            onClick <| mapMsg <| ElementExpand item (not expanded)
    in
    column
        [ width (maximum 1000 fill)
        , height fill
        , padding 20
        , spacing 20
        , centerX
        ]
        [ map mapMsg (viewFilter search)
        , column [ width fill ]
            (List.indexedMap
                (\i ( expanded, item ) ->
                    el
                        [ width fill, Element.Background.color (bg i), padding 10 ]
                        (viewElement (expand item expanded) expanded item)
                )
                (List.filter (filter search << Tuple.second) items)
            )
        , map mapMsg (viewAdd add)
        ]


update : ExpandableListMsg a elementMsg -> ExpandableList a msg elementMsg -> ( ExpandableList a msg elementMsg, Cmd msg )
update msg model =
    case msg of
        InputChanged string ->
            ( { model | search = string }, Cmd.none )

        ElementMsg element eMsg ->
            let
                ( new, cmd ) =
                    model.update eMsg element
            in
            ( { model
                | items =
                    List.map
                        (\( ex, e ) ->
                            if e == element then
                                ( ex, new )

                            else
                                ( ex, e )
                        )
                        model.items
              }
            , cmd
            )

        ElementExpand element isExpanded ->
            let
                expandElement =
                    \( ex, e ) ->
                        if e == element then
                            ( isExpanded, e )

                        else
                            ( ex, e )

                newModel =
                    { model | items = List.map expandElement model.items }
            in
            Maybe.withDefault ( newModel, Cmd.none )
                (Maybe.map (\m -> update (ElementMsg element m) newModel) model.expandItem)

        AddElement ->
            case model.add of
                Just f ->
                    ( { model | items = model.items ++ [ ( True, f () ) ] }, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

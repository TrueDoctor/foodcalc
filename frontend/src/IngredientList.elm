module IngredientList exposing (..)

import Element exposing (..)
import Element.Background
import Element.Input
import Html.Attributes exposing (name)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg)
import Platform.Cmd as Cmd
import Element.Border


type alias IngredientsList =
    ExpandableList Ingredient IngredientListMsg IngredientMsg


type Ingredients
    = Ingredients IngredientsList


type alias IngredientData =
    { name : String, id : Int, energy : String, comment : Maybe String }


type Ingredient
    = Ingredient { data : IngredientData, edit : IngredientData }


type IngredientMsg
    = Save
    | NameChange String
    | EnergyChange String
    | CommentChange String


type IngredientListMsg
    = ListMsg (ExpandableListMsg Ingredient IngredientMsg)


init : ExpandableList Ingredient IngredientListMsg IngredientMsg
init =
    let
        filter : String -> Ingredient -> Bool
        filter string ingredient =
            case ingredient of
                Ingredient { data } ->
                    String.contains (String.toLower string) (String.toLower data.name)
    in
    { search = ""
    , filter = filter
    , items = [ ( False, newIngredient 1 "test" "1" (Just "") ) ]
    , viewElement = viewIngredient
    , mapMsg = ListMsg
    , update = updateIngredient
    , add =
        Just
            (\() ->
                newIngredient -1 "" "" (Just "")
            )
    }


newIngredient : Int -> String -> String -> Maybe String -> Ingredient
newIngredient id name energy comment =
    Ingredient { data = { id = id, name = name, energy = energy, comment = comment }, edit = { id = id, name = name, energy = energy, comment = comment } }


viewExpanded ingredient =
    case ingredient of
        Ingredient { edit } ->
            column [ Element.Background.color (rgb 1 1 1), width fill, padding 10, spacing 10, Element.Border.rounded 5 ]
                [ row
                    [ width fill
                    , spacing 25
                    ]
                    [ Element.map
                        (ListMsg << ExpandableList.mapElementMsg ingredient)
                        (Element.Input.text []
                            { onChange = NameChange
                            , label = Element.Input.labelLeft [] (text "Name:")
                            , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                            , text = edit.name
                            }
                        )
                    , Element.map
                        (ListMsg << ExpandableList.mapElementMsg ingredient)
                        (Element.Input.text []
                            { onChange = EnergyChange
                            , label = Element.Input.labelLeft [] (text "Energy:")
                            , placeholder = Just (Element.Input.placeholder [] (text "Energy"))
                            , text = edit.energy
                            }
                        )
                    ]
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg ingredient)
                    (Element.Input.text []
                        { onChange = CommentChange
                        , label = Element.Input.labelLeft [] (text "Comment:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                        , text = Maybe.withDefault "" edit.comment
                        }
                    )
                , row [ width fill, spacing 25 ]
                    [ Element.Input.button [ alignRight ] { onPress = Nothing, label = el [ padding 10 ] <| text "Save" }
                    , Element.Input.button [ alignRight ] { onPress = Nothing, label = el [ padding 10 ] <| text "Cancel" }
                    ]
                ]


viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20 ]
        [ el [ width (fillPortion 1) ] (text (String.fromInt data.id))
        , el [ width (fillPortion 4) ] (text data.name)
        , el [ width (fillPortion 2) ] (text data.energy)
        , el [ width (fillPortion 5) ] (text (Maybe.withDefault "" data.comment))
        ]


viewIngredient : Bool -> Ingredient -> Element IngredientListMsg
viewIngredient expanded ingredient =
    case ingredient of
        Ingredient { data } ->
            column [ width fill ]
                [ viewRow data
                , if expanded then
                    viewExpanded ingredient

                  else
                    none
                ]


view : IngredientsList -> Element IngredientListMsg
view model =
    ExpandableList.view model


update : IngredientListMsg -> IngredientsList -> ( IngredientsList, Cmd IngredientListMsg )
update msg model =
    case msg of
        ListMsg m ->
            ExpandableList.update m model


updateIngredient : IngredientMsg -> Ingredient -> ( Ingredient, Cmd IngredientListMsg )
updateIngredient msg ig =
    let 
        updateEdit f =
            case ig of 
                Ingredient i ->
                    let edit = i.edit in 
                        Ingredient {i | edit = f edit}
    in
    case msg  of
        NameChange name  ->
            (updateEdit (\e -> {e|name=name}), Cmd.none)
        CommentChange comment ->
            (updateEdit(\e->{e|comment = Just comment}), Cmd.none)
        EnergyChange energy ->
            (updateEdit (\e->{e|energy = energy}),Cmd.none)
        Save -> 
            (ig, Debug.todo "Copy DB connection")

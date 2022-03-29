import styled from 'styled-components'
import { PageText as NavText } from '../helpers/PageText'
import { PageItemWrapper as NavItemWrapper } from '../helpers/ItemWrapper'

export const Container = styled.div`
    display: flex;
    justify-content: space-evenly;
    align-items: center;
    color: white;
    background-color: #131a22;
`
export const Logo = styled.img`
    width: 6em;
    border: 1px solid #131a22;
    padding: 0.2em 0.1em;
    cursor: pointer;
    &:hover {
        border: 1px solid #ffffff;
        border-radius: 0.2em;
    }
`
export const Text = styled(NavText)`
    color: ${(props) => (props.color ? props.color : '#ffffff')};
    font-size: ${(props) => (props.fontSize ? props.fontSize : '.9em')};
`

export const Wrapper = styled(NavItemWrapper)`
    display: flex;
    flex-direction: ${(props) => (props.flexDirection ? props.flexDirection : 'column')};
    align-items: ${(props) => (props.alignItems ? props.alignItems : 'flex-start')};
    padding: 0.1em;
    cursor: pointer;
    border: 1px solid #131a22;
    &:hover {
        border: 1px solid #ffffff;
        border-radius: 0.2em;
    }
    @media (max-width: 850px) {
        display: none;
    }
`
